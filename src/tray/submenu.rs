use super::{menu::RootMenu, touch_or_init_cached_box, tray, tray_menu_widget, TrayIcon};
use colored::Colorize;
use gtk4::{prelude::*, Box, Orientation::Horizontal};
use system_tray::menu::MenuItem;

pub trait Submenu {
    fn add_submenu(self: &Self, id: i32, label: String, submenu: Vec<MenuItem>);
    fn connect_back_button(self: &Self, back_button: &gtk4::Button, cache_key: String);
    fn connect_submenu_activate(self: &Self, label: String, menu_item_button: &gtk4::Button);
}

impl Submenu for TrayIcon {
    fn add_submenu(self: &Self, id: i32, label: String, submenu: Vec<MenuItem>) {
        touch_or_init_cached_box(
            &format!("{}/menu/0/submenu/{}", self.address, label).to_string(),
            &label,
            |submenu_widget, cache_key| {
                let icon = gtk4::Image::from_icon_name("go-previous-symbolic");
                let label = gtk4::Label::builder().label(label.as_str()).build();
                let row = Box::builder().orientation(Horizontal).build();
                row.append(&icon);
                row.append(&label);
                let back_button = gtk4::Button::builder().build();
                back_button.set_child(Some(&row));
                back_button.add_css_class("back");
                self.connect_back_button(&back_button, cache_key.clone());
                submenu_widget.prepend(&back_button);
            },
            move |submenu_widget, _| self.add_submenu_items(id, submenu_widget, submenu),
        );
    }

    fn connect_back_button(self: &Self, back_button: &gtk4::Button, cache_key: String) {
        let address = format!("{}", self.address);
        back_button.connect_clicked(move |_| {
            eprintln!(
                "[{}]: {}({})",
                "tray".green(),
                "close_submenu".yellow(),
                cache_key,
            );
            let cache_key = format!("{}/menu/0", address);
            match tray_menu_widget().get(&cache_key) {
                Some(tray_menu_widget) => {
                    let item = tray().items.get(&address).unwrap();
                    item.button
                        .popover()
                        .unwrap()
                        .set_child(Some(tray_menu_widget.as_ref()));
                    return;
                }
                None => (),
            }
        });
    }

    fn connect_submenu_activate(self: &Self, label: String, menu_item_button: &gtk4::Button) {
        let address = format!("{}", self.address);
        let cache_key = format!("{}/menu/0/submenu/{}", address, label);
        menu_item_button.connect_clicked(move |_| {
            eprintln!(
                "[{}]: {}({})",
                "tray".green(),
                "open_submenu".yellow(),
                cache_key
            );
            match tray_menu_widget().get(&cache_key) {
                Some(tray_menu_widget) => {
                    let item = tray().items.get(&address).unwrap();
                    item.button
                        .popover()
                        .unwrap()
                        .set_child(Some(tray_menu_widget.as_ref()));
                    return;
                }
                None => (),
            }
        });
    }
}
