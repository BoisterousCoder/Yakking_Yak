use ::gtk::{prelude::*, Button, CheckButton, Entry, Text};
use adw::{ApplicationWindow, HeaderBar};
use ::gtk::{Box, ListBox, Orientation, SelectionMode, ScrolledWindow, MenuButton, Popover};
use glib::{self, timeout_add_local};
use std::time::Duration;

use crate::{on_join_group, on_send_msg, SOCKET_CLIENT, update_msg_display, SLEEP_DURATION, APP_TITLE, MSG_QUEUE, STATE};
use crate::client::{serverhandlers::{ServerMsg, MsgContent}, utils::log};

use super::sign_in::on_sign_in;

pub fn build_sign_in(app: &adw::Application) {
    let content = Box::new(Orientation::Vertical, 0);
    content.append(&HeaderBar::new());

    let sign_in_content = Box::new(Orientation::Vertical, 0);

    let username_label = Text::builder()
        .text("Username:")
        .editable(false)
        .build();
    sign_in_content.append(&username_label);
    let username_entry = Entry::builder()
        .margin_bottom(2)
        .name("Username")
        .build();
    username_entry.connect_activate(|username_entry| {
        let sign_in_button = username_entry
            .parent().unwrap()
            .last_child().unwrap()
            .prev_sibling().unwrap()
            .downcast().expect("Found UI emlement but is not button! UI is broke pls fix");
        on_sign_in(&sign_in_button);
    });
    sign_in_content.append(&username_entry);

    let password_label = Text::builder()
        .text("Password:")
        .editable(false)
        .build();
    sign_in_content.append(&password_label);
    let password_entry = Entry::builder()
        .margin_bottom(2)
        .name("Password")
        .input_purpose(gtk::InputPurpose::Password)
        .build();
    password_entry.connect_activate(|username_entry| {
        let sign_in_button = username_entry
            .parent().unwrap()
            .last_child().unwrap()
            .prev_sibling().unwrap()
            .downcast().expect("Found UI emlement but is not button! UI is broke pls fix");
        on_sign_in(&sign_in_button);
    });
    sign_in_content.append(&password_entry);

    let sign_in_button = Button::builder()
        .margin_bottom(5)
        .label("Sign In")
        .build();
    sign_in_button.connect_clicked(on_sign_in);
    sign_in_content.append(&sign_in_button);

    let errors_label = Text::builder()
        .text("")
        .editable(false)
        .build();
    sign_in_content.append(&errors_label);
    
    content.append(&sign_in_content);

    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .default_width(350)
        .content(&content)
        .build();

    window.show();
}

pub fn build_content(content: &Box){
    let app_content = Box::new(Orientation::Vertical, 0);

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

    let user_list = Popover::builder()
        .autohide(true)
        .halign(gtk::Align::Start)
        .build();
    let user_list_button = MenuButton::builder()
        .direction(gtk::ArrowType::Down)
        .halign(gtk::Align::Start)
        .label("Online Users")
        .popover(&user_list)
        .build();
    upper_left_box.append(&user_list_button);

    upper_row.append(&upper_left_box);

    let upper_right_box = Box::builder()
        .orientation(Orientation::Horizontal)
        .hexpand(true)
        .build();
    // let group_entry_label = Text::builder()
    //     .text("Group: ")
    //     .editable(false)
    //     .build();
    // upper_right_box.append(&group_entry_label);
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
            // .next_sibling().unwrap()
            .downcast().expect("Found UI emlement but is not entry! UI is broke pls fix");
        on_join_group(&group_entry)
    });
    upper_right_box.append(&join_button);

    upper_row.append(&upper_right_box);
    app_content.append(&upper_row);

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

    app_content.append(&scroll_box);

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

    app_content.append(&bottom_row);
    
    content.append(&app_content);
    
    timeout_add_local( Duration::from_millis(SLEEP_DURATION), move || {
        while let Some(txt) = MSG_QUEUE.pop() {
            log("handing msg");
            let state = &mut STATE.lock().expect("unable to aquire state");

            if let Some(msg) = ServerMsg::from_server(&txt, state){
                update_msg_display(&msg_list, &user_list, state);

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