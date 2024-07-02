use super::{menu::RootMenu, touch_or_init_cached_box, tray, tray_menu_widget, TrayIcon};
use colored::Colorize;
use gtk4::{prelude::*, Box, Orientation::Horizontal};
use system_tray::menu::MenuItem;

pub trait Submenu {
    fn add_submenu(self: &Self, id: i32, label: String, submenu: Vec<MenuItem>);
    fn connect_back_button(self: &Self, back_button: &gtk4::Button, id: i32);
    fn connect_submenu_activate(self: &Self, id: i32, menu_item_button: &gtk4::Button);
}

impl Submenu for TrayIcon {
    fn add_submenu(self: &Self, id: i32, label: String, submenu: Vec<MenuItem>) {
        touch_or_init_cached_box(
            &format!("{}/menu/0/submenu/{}", self.address, id),
            label.as_str(),
            |submenu_widget| {
                let icon = gtk4::Image::from_icon_name("go-previous-symbolic");
                let label = gtk4::Label::builder().label(label.as_str()).build();
                let row = Box::builder().orientation(Horizontal).build();
                row.append(&icon);
                row.append(&label);
                let back_button = gtk4::Button::builder().build();
                back_button.set_child(Some(&row));
                back_button.add_css_class("back");
                self.connect_back_button(&back_button, id);
                submenu_widget.prepend(&back_button);
            },
            move |submenu_widget| self.add_submenu_items(id, submenu_widget, submenu),
        );
    }

    fn connect_back_button(self: &Self, back_button: &gtk4::Button, id: i32) {
        let address = format!("{}", self.address);
        back_button.connect_clicked(move |_| {
            eprintln!(
                "[{}]: {}({},{})",
                "tray".green(),
                "close_submenu".yellow(),
                address,
                id
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

    fn connect_submenu_activate(self: &Self, id: i32, menu_item_button: &gtk4::Button) {
        let menu_path = format!("{}", self.menu_path);
        let address = format!("{}", self.address);
        menu_item_button.connect_clicked(move |_| {
            eprintln!(
                "[{}]: {}({},{})",
                "tray".green(),
                "open_submenu".yellow(),
                menu_path,
                id
            );
            let cache_key = format!("{}/menu/0/submenu/{}", address, id);
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
