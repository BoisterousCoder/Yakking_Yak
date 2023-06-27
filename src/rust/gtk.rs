#![cfg(not(target_arch = "wasm32"))]

//use gtk::prelude::*;
use gtk::{glib::ExitCode, prelude::*, Button, Text, CheckButton, Entry};
use adw::{ActionRow, Application, ApplicationWindow, HeaderBar, traits::ActionRowExt};
use gtk::{Box, ListBox, Orientation, SelectionMode};

static APP_ID: &str = "com.BoisterousCoder.YakkingYak";
static APP_TITLE: &str = "Yakking Yak";

fn main() -> ExitCode {
    //gio::resources_register_include!("todo_5.gresource").expect("Failed to register resources.");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    // Connect to signals
    app.connect_startup(setup_shortcuts);
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn setup_shortcuts(app: &adw::Application) {
    // app.set_accels_for_action("win.filter('All')", &["<Ctrl>a"]);
    // app.set_accels_for_action("win.filter('Open')", &["<Ctrl>o"]);
    // app.set_accels_for_action("win.filter('Done')", &["<Ctrl>d"]);
}


fn build_ui(app: &adw::Application){
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
    upper_right_box.append(&group_entry);

    let join_button = Button::builder()
        .label("Join")
        .halign(gtk::Align::End)
        .margin_start(8)
        .build();
    upper_right_box.append(&join_button);

    upper_row.append(&upper_right_box);
    content.append(&upper_row);

    let msg_list = ListBox::builder()
        .margin_top(16)
        .margin_end(24)
        .margin_start(24)
        .margin_bottom(16)
        .vexpand(true)
        .selection_mode(SelectionMode::None)
        .overflow(gtk::Overflow::Visible)
        // makes the list look nicer
        .css_classes(vec![String::from("boxed-list")])
        .build();
    content.append(&msg_list);

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
    bottom_row.append(&msg_entry);

    let send_button = Button::builder()
        .label("Send")
        .halign(gtk::Align::End)
        .margin_end(8)
        .margin_start(8)
        .build();
    send_button.connect_clicked(|_| {
        eprintln!("Clicked!");
    });
    bottom_row.append(&send_button);

    let encryption_toggle = CheckButton::builder()
        .label("Encryption")
        .halign(gtk::Align::End)
        .build();
    bottom_row.append(&encryption_toggle);

    content.append(&bottom_row);
    // let row = ActionRow::builder()
    //     .activatable(true)
    //     .title("Click me")
    //     .build();
    // row.connect_activated(|old_row| {
    //     let new_row = ActionRow::builder()
    //         .activatable(false)
    //         .title("Don't click me")
    //         .build();
    //     let list = old_row.parent().expect("UI is broke pls fix") as ListBox;
    //     list.append(&new_row);
    // });
    // list.append(&row);
    
    let window = ApplicationWindow::builder()
        .application(app)
        .title(APP_TITLE)
        .default_width(350)
        .content(&content)
        .build();

    window.show();
}