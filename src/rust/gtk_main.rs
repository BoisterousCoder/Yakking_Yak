#![cfg(not(target_arch = "wasm32"))]

use crate::client_gtk::build_ui::build_sign_in;
use ::gtk::{glib::ExitCode, prelude::*, Button, CheckButton, Entry};
use adw::{ActionRow, Application};
use ::gtk::{Box, ListBox, Orientation, Popover};
use rand_core::{OsRng, RngCore};
use crate::client_gtk::save::GroupSave;
use client::utils::Address;
use std::sync::{Mutex, Arc};
use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload};
use crossbeam_queue::SegQueue;

#[macro_use]
extern crate lazy_static;

mod client;
mod client_gtk;

use crate::client::store::Crypto;
use crate::client::serverhandlers::{ServerMsg, MsgContent};
use crate::client::utils::log;

static APP_ID: &str = "com.BoisterousCoder.YakkingYak";
static APP_TITLE: &str = "Yakking Yak";

const SLEEP_DURATION:u64 = 1500;//In mils
const SEED:u64 = 1234567890; //TODO: fix the seed to its actually random
const PASSWORD:&str = "ABCDE";
const PROXY_SEED:u64 = 0987654321; //TODO: fix the seed to its actually random
const DEVICE_ID:[u8; 32] = [1u8; 32];//TODO: Make this useful
const MSG_TYPES:[char; 6] = ['i', 's', 't', 'l', 'p', 'j'];
const SOCKET_SERVER_ADDRESS:&'static str = "http://localhost:4000";
const IS_AUTO_SAVING:bool = true;

struct SignInDetails {
    pub private_device_id:[u8; 32],
    pub username:String,
    pub password:String
}

lazy_static! {
    static ref GROUP:Mutex<String> = {
        Mutex::new("".to_string())
    };
    static ref STATE:Mutex<Crypto> = {
        let user_number:u32 = OsRng.next_u32();
        let user_name = format!("Anon{:X}", user_number);
        Mutex::new(Crypto::new(&user_name, PASSWORD, DEVICE_ID, OsRng.next_u64(), OsRng.next_u64()))
    };
    pub static ref MSG_QUEUE:SegQueue::<String> = SegQueue::new();
    pub static ref SOCKET_CLIENT:Arc<Client> = {
        let on_msg = move |payload_wrapped, _| {
            if let Payload::String(payload) = payload_wrapped {
                log(&format!("Recieved Msg {}", &payload));
                let payload_fixed = payload.replace("\"", "");
                MSG_QUEUE.push(payload_fixed);
            };
        };
    
        let mut socket_builder = ClientBuilder::new(SOCKET_SERVER_ADDRESS).namespace("/");
        for msg_type in MSG_TYPES {
            socket_builder = socket_builder.on(msg_type.to_string(), on_msg.clone());
        }
    
        Arc::new(socket_builder.connect().expect("Unable to connect to server"))
    }; 
}

fn main() -> ExitCode {
    //The first emit doesn't seem to run right. This is a bypass for this.
    SOCKET_CLIENT.emit("TEST", "WARMUP").unwrap();

    let app = Application::builder()
        .application_id(APP_ID)
        .build();
    
    app.connect_activate(build_sign_in);

    app.run()
}

fn on_join_group(group_entry:&Entry){
    let state = &mut STATE.lock().unwrap();

    let group = &mut GROUP.lock().unwrap();
    if !group.is_empty(){
        let old_save = state.group_as_save(&group);
        old_save.save(&state.password).expect("Unable to save the current group");
    }
    
    let new_group = group_entry.buffer().text().to_string();
    **group = new_group;
    
    if let Some(save) = GroupSave::load(state.get_address(), &group, &state.password){
        state.load_group_save(save);
        log("Successfully loaded group")
    }else{
        state.new_group(SEED, PROXY_SEED);
        log("Changed to new group");
    }
    let content = MsgContent::Join(group.to_string());
    let msg =  ServerMsg::new(&state.get_address(), content);
    SOCKET_CLIENT.emit("j", msg.to_string(&state)).expect("failed to send join message");
}
fn on_send_msg(msg_entry:&Entry){
    let state = &mut STATE.lock().unwrap();
    let text = msg_entry.buffer().text().to_string();
    if text.is_empty(){
        return ();
    }
    let encyption_checkbox:CheckButton = msg_entry.next_sibling().unwrap()
        .next_sibling().unwrap()
        .downcast().expect("Found UI emlement but is not checkbutton! UI is broke pls fix");
    
    let (content, label) = if encyption_checkbox.is_active(){
        let encrypted_text = state.encrypt(text);
        (MsgContent::SecureText(encrypted_text), "s")
    }else {
        (MsgContent::InsecureText(text), "i")
    };
    let msg = ServerMsg::new(&state.get_address(), content);

    SOCKET_CLIENT.emit(label, msg.to_string(&state)).expect("failed to send join message");
    msg_entry.buffer().set_text("");
}
fn update_msg_display(msg_list:&ListBox, user_list:&Popover, state:&Crypto){
    while let Some(child) = msg_list.first_child() {
        msg_list.remove(&child);
    }
    for msg in state.get_msgs(){
        display_msg(msg_list, msg, state);
    }
    
    let user_list_box = Box::new(Orientation::Vertical, 5);
    for agent in state.get_agents(){
        if agent.is_online{
            let relation = state.relation(&agent.keys.address);
            let relation_display = if &relation == "self" || &relation == "trusted" {
                relation
            }else{
                "untrusted".to_string()
            };
            let user_display = format!("{}--{}",
                relation_display,
                agent.keys.address.name);
            let user_button = Button::builder()
                .label(user_display)
                .halign(::gtk::Align::Start)
                .hexpand(true)
                .build();
            user_button.connect_clicked(move |_| on_user_click(&agent.keys.address));
            user_list_box.append(&user_button);
        }
    }
    user_list.set_child(Some(&user_list_box));
    
    if IS_AUTO_SAVING{
        state.group_as_save(&GROUP.lock().unwrap())
            .save(&state.password)
            .expect("Failed to autosave!");
    }
}
fn display_msg(msg_list:&ListBox, msg:ServerMsg, state:&Crypto){
    if let Some(msg_display) = msg.display(state){
        let mut msg_parts = msg_display.split("\r").into_iter();
        let name_plate = msg_parts.next().unwrap();
        let content = msg_parts.next().unwrap();
        let relation = msg_parts.next().unwrap();

        let row = ActionRow::builder()
            .title(content)
            .subtitle(name_plate)
            .css_classes(vec![relation.to_string()])
            .activatable(true)
            .build();
        msg_list.append(&row);
    }else{
        log("Message recieved but there's nothing to display")
    }
}

fn on_user_click(from:&Address){
    let state = &mut STATE.lock().unwrap();
    if state.relation(from) == "allowedTrust".to_string(){
        let content = match state.trust(from.name.to_string()) {
            Some(forein) => Some(MsgContent::Trust(forein.clone())),
            None => None
        };
        if content.is_some() {
            let msg = ServerMsg::new(&state.get_address(), content.unwrap());
            SOCKET_CLIENT.emit("t",msg.to_string(&state)).expect("failed to send join message");
        }
    }else{
        log(&format!("Can't trust {} because you already trust them, you dont have their primary key, or it's you. Can't trust yourself after all.", from.name))
    }
}