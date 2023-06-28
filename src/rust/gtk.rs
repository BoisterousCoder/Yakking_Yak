#![cfg(not(target_arch = "wasm32"))]

use gtk::{glib::ExitCode, prelude::*, Button, Text, CheckButton, Entry};
use adw::{ActionRow, Application, ApplicationWindow, HeaderBar, ClampScrollable};
use gtk::{Box, ListBox, Orientation, SelectionMode, ScrolledWindow};
use rand_core::{RngCore, OsRng};
use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;

mod utils;
mod store;
mod ratchet;
mod serverhandlers;
mod KeyBundle;
mod ForeinAgent;

use crate::store::Crypto;
use crate::serverhandlers::{ServerMsg, MsgContent};
use crate::utils::log;

static APP_ID: &str = "com.BoisterousCoder.YakkingYak";
static APP_TITLE: &str = "Yakking Yak";

const RANDOM_NUMBER:u64 = 1234567890; //TODO: fix the seed to its actually random
const DEVICE_ID:i32 = 12345;//TODO: Make this useful

lazy_static! {
    static ref STATE:Mutex<Crypto> = {
        let user_number:u32 = OsRng.next_u32();
        let user_name = format!("Anon{:X}", user_number);
        Mutex::new(Crypto::new(&user_name, DEVICE_ID, RANDOM_NUMBER))
    };
}

fn main() -> ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    // Connect to signals
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &adw::Application){
    //let mut rng = OsRng::default();
    

    let content = Box::new(Orientation::Vertical, 0);
    content.append(&HeaderBar::new());
    // content.set_("state", state);

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
        .text("Yakking Yak")
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
        .build();
    group_entry.connect_activate(|group_entry| on_join_group(group_entry, &mut STATE.lock().unwrap()));
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
        on_join_group(&group_entry, &mut STATE.lock().unwrap())
    });
    upper_right_box.append(&join_button);

    upper_row.append(&upper_right_box);
    content.append(&upper_row);

    let msg_list = ListBox::builder()
        .vexpand(true)
        .selection_mode(SelectionMode::None)
        .css_classes(vec![String::from("boxed-list")])
        .build();

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
    msg_entry.connect_activate(|msg_entry| on_send_msg(msg_entry, &mut STATE.lock().unwrap()));
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
        on_send_msg(&msg_entry, &mut STATE.lock().unwrap())
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
}
fn on_join_group(group_entry:&Entry, state:&mut Crypto){
    log(&group_entry.buffer().text());
    group_entry.buffer().set_text("");
    log(&state.get_address().name);
}
fn on_send_msg(msg_entry:&Entry, state:&mut Crypto){
    let text = msg_entry.buffer().text().to_string();
    let msg = ServerMsg::new(&state.get_address(), MsgContent::InsecureText(text));
    
    let list:ListBox = msg_entry.parent().unwrap()
        .prev_sibling().unwrap()
        .first_child().unwrap()
        .first_child().unwrap()
        .downcast().expect("Found UI emlement but is not list! UI is broke pls fix");
    display_msg(&list, msg, state);

    msg_entry.buffer().set_text("");
    log(&state.get_address().name);
}
fn display_msg(list:&ListBox, msg:ServerMsg, state:&mut Crypto){
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
    }
}