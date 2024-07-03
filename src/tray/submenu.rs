use std::sync::Arc;

use super::{menu::RootMenu, touch_or_init_cached_box, TrayIcon};
use colored::Colorize;
use gtk4::{prelude::*, Box, Orientation::Horizontal};
use system_tray::menu::MenuItem;

pub trait Submenu {
    fn add_submenu(self: &Self, id: i32, label: String, submenu: Vec<MenuItem>);
    fn connect_activate_menu(self: &Self, button: &gtk4::Button, cache_key: String);
    fn connect_back_button(self: &Self, back_button: &gtk4::Button, cache_key: String);
    fn connect_submenu_activate(self: &Self, label: String, menu_item_button: &gtk4::Button);
}

impl Submenu for TrayIcon {
    fn add_submenu(self: &Self, id: i32, label: String, submenu: Vec<MenuItem>) {
        touch_or_init_cached_box(
            &format!("{}/menu/0/submenu/{}", self.address, label).to_string(),
            &label,
            |rows, cache_key| {
                let icon = gtk4::Image::from_icon_name("go-previous-symbolic");
                let label = gtk4::Label::builder().label(label.as_str()).build();
                let row = Box::builder().orientation(Horizontal).build();
                row.append(&icon);
                row.append(&label);
                let back_button = gtk4::Button::builder().build();
                back_button.set_child(Some(&row));
                back_button.add_css_class("back");
                self.connect_back_button(&back_button, cache_key.clone());
                rows.prepend(&back_button);
                self.stack
                    .add_named(rows.as_ref(), Some(cache_key.as_str()));
            },
            move |submenu_widget, _| self.add_submenu_items(id, submenu_widget, submenu),
        );
    }

    fn connect_back_button(self: &Self, back_button: &gtk4::Button, _: String) {
        let cache_key = format!("{}/menu/0", self.address);
        self.connect_activate_menu(back_button, cache_key);
    }

    fn connect_submenu_activate(self: &Self, label: String, menu_item_button: &gtk4::Button) {
        let cache_key = format!("{}/menu/0/submenu/{}", self.address, label);
        self.connect_activate_menu(menu_item_button, cache_key);
    }

    fn connect_activate_menu(self: &Self, button: &gtk4::Button, cache_key: String) {
        let stack = Arc::clone(&self.stack);
        button.connect_clicked(move |_| {
            eprintln!(
                "[{}]: {}({})",
                "tray".green(),
                "show_menu".yellow(),
                cache_key,
            );
            stack.set_visible_child_name(&cache_key);
        });
    }
}
