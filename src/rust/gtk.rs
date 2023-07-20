#![cfg(not(target_arch = "wasm32"))]

use adw::traits::ActionRowExt;
use gtk::{glib::ExitCode, prelude::*, Button, Text, CheckButton, Entry};
use adw::{ActionRow, Application, ApplicationWindow, HeaderBar};
use gtk::{Box, ListBox, Orientation, SelectionMode, ScrolledWindow};
use glib::{self, timeout_add_local};
use lib::utils::Address;
use rand_core::{RngCore, OsRng};
use std::sync::{Mutex, Arc};
use std::time::Duration;
use rust_socketio::client::Client;
use rust_socketio::{ClientBuilder, Payload};
use crossbeam_queue::SegQueue;

#[macro_use]
extern crate lazy_static;

mod lib;

use crate::lib::store::Crypto;
use crate::lib::serverhandlers::{ServerMsg, MsgContent};
use crate::lib::utils::log;

static APP_ID: &str = "com.BoisterousCoder.YakkingYak";
static APP_TITLE: &str = "Yakking Yak";

const SLEEP_DURATION:u64 = 1500;//In mils
const RANDOM_NUMBER:u64 = 1234567890; //TODO: fix the seed to its actually random
const DEVICE_ID:i32 = 12345;//TODO: Make this useful
const MSG_TYPES:[char; 6] = ['i', 's', 't', 'l', 'p', 'j'];
const SOCKET_SERVER_ADDRESS:&'static str = "http://localhost:4000";

lazy_static! {
    static ref STATE:Mutex<Crypto> = {
        let user_number:u32 = OsRng.next_u32();
        let user_name = format!("Anon{:X}", user_number);
        Mutex::new(Crypto::new(&user_name, DEVICE_ID, RANDOM_NUMBER))
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
    // static ref SOCKET_CLIENT:Mutex<Option<Client>> = Mutex::new(None);
}

fn main() -> ExitCode {
    SOCKET_CLIENT.emit("TEST", "WARMUP").unwrap();

    let app = Application::builder()
        .application_id(APP_ID)
        .build();
    
    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &adw::Application){
    let state = &STATE.lock().unwrap();
    let username = state.get_address().name;
    std::mem::drop(state);

    let msg_list = ListBox::builder()
    .vexpand(true)
    .selection_mode(SelectionMode::None)
    .css_classes(vec![String::from("boxed-list")])
    .build();

    let content = Box::new(Orientation::Vertical, 0);
    content.append(&HeaderBar::new());

    let upper_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(16)
        .margin_end(24)
        .margin_start(24)
        .hexpand(true)
        .build();
    
    let upper_left_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();

    let header_text = Text::builder()
        .editable(false)
        .can_target(false)
        .can_focus(false)
        .text(username)
        .build();
    upper_left_box.append(&header_text);

    upper_row.append(&upper_left_box);

    let upper_right_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();
    
    let group_entry = Entry::builder()
        .hexpand(true)
        .placeholder_text("Group")
        .text("Default")
        .build();
    group_entry.connect_activate(on_join_group);
    upper_right_box.append(&group_entry);

    let join_button = Button::builder()
        .label("Join")
        .halign(gtk::Align::End)
        .margin_start(8)
        .build();
    join_button.connect_clicked(|button| {
        let group_entry:Entry = button
            .parent().unwrap()
            .first_child().unwrap()
            .downcast().expect("Found UI emlement but is not entry! UI is broke pls fix");
        on_join_group(&group_entry)
    });
    upper_right_box.append(&join_button);

    upper_row.append(&upper_right_box);
    content.append(&upper_row);

    let scroll_box = ScrolledWindow::builder()
        .margin_top(16)
        .margin_end(24)
        .margin_start(24)
        .margin_bottom(16)
        .child(&msg_list)
        .build();

    content.append(&scroll_box);

    let bottom_row = Box::builder()
        .orientation(Orientation::Horizontal)
        .margin_end(24)
        .margin_start(24)
        .margin_bottom(16)
        .hexpand(true)
        .build();

    let msg_entry = Entry::builder()
        .hexpand(true)
        .placeholder_text("Enter Message")
        .build();
    msg_entry.connect_activate(on_send_msg);
    bottom_row.append(&msg_entry);

    let send_button = Button::builder()
        .label("Send")
        .halign(gtk::Align::End)
        .margin_end(8)
        .margin_start(8)
        .build();
    send_button.connect_clicked(|button| {
        let msg_entry:Entry = button
            .parent().unwrap()
            .first_child().unwrap()
            .downcast().expect("Found UI emlement but is not entry! UI is broke pls fix");
        on_send_msg(&msg_entry)
    });
    bottom_row.append(&send_button);

    let encryption_toggle = CheckButton::builder()
        .label("Encryption")
        .halign(gtk::Align::End)
        .build();
    bottom_row.append(&encryption_toggle);

    content.append(&bottom_row);
    
    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .default_width(350)
        .content(&content)
        .build();

    window.show();
    
    timeout_add_local( Duration::from_millis(SLEEP_DURATION), move || {
        while let Some(txt) = MSG_QUEUE.pop() {
            log("handing msg");
            let state = &mut STATE.lock().expect("unable to aquire state");

            if let Some(msg) = ServerMsg::from_server(&txt, state){
                update_msg_display(&msg_list, state);

                if let MsgContent::Join(_) = msg.content {
                    let content_to_send = MsgContent::PublicKey(state.public_key());
                    let msg_to_send = ServerMsg::new(&state.get_address(), content_to_send);
                    SOCKET_CLIENT.emit("p", msg_to_send.to_string(&state)).expect("unable to send primary keys");
                }
            }
        };
        return glib::source::Continue(true);
    });
}

fn on_join_group(group_entry:&Entry){
    let state = &mut STATE.lock().unwrap();
    state.empty_msgs();

    log("Group Change");
    let content = MsgContent::Join(group_entry.buffer().text().to_string());
    let msg =  ServerMsg::new(&state.get_address(), content);
    SOCKET_CLIENT.emit("j", msg.to_string(&state)).expect("failed to send join message");

    //TODO: Drop all previous messages
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
fn update_msg_display(list:&ListBox, state:&Crypto){
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    for msg in state.get_msgs(){
        display_msg(list, msg, state);
    }
}
fn display_msg(list:&ListBox, msg:ServerMsg, state:&Crypto){
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
        list.append(&row);
        row.connect_activated(move |_| on_msg_click(&msg.from));
    }else{
        log("Message recieved but there's nothing to display")
    }
}

fn on_msg_click(from:&Address){
    let state = &mut STATE.lock().unwrap();
    log(&format!("clicked on {}\n All people known:\n{}", from.name, state.list_people()));
    if state.relation(from) == "allowedTrust".to_string(){
        let content = match state.trust(from.name.to_string()) {
            Some(forein) => Some(MsgContent::Trust(forein.clone())),
            None => None
        };
        if content.is_some() {
            let msg = ServerMsg::new(&state.get_address(), content.unwrap());
            SOCKET_CLIENT.emit("t",msg.to_writable(&state)).expect("failed to send join message");
        }
    }else{
        log(&format!("Can't trust {} because you already trust them, you dont have their primary key, or it's you. Can't trust yourself after all.", from.name))
    }
}