use super::{menu_item::MenuItem as TrayMenuItem, section::*, tray_item::*};
use colored::Colorize;
use gtk4::{prelude::*, Box};
use std::sync::Arc;
use system_tray::menu::{MenuItem, TrayMenu};

/*
 ████████╗██████╗  █████╗ ██╗   ██╗    ██████╗  ██████╗  ██████╗ ████████╗    ███╗   ███╗███████╗███╗   ██╗██╗   ██╗
 ╚══██╔══╝██╔══██╗██╔══██╗╚██╗ ██╔╝    ██╔══██╗██╔═══██╗██╔═══██╗╚══██╔══╝    ████╗ ████║██╔════╝████╗  ██║██║   ██║
    ██║   ██████╔╝███████║ ╚████╔╝     ██████╔╝██║   ██║██║   ██║   ██║       ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║
    ██║   ██╔══██╗██╔══██║  ╚██╔╝      ██╔══██╗██║   ██║██║   ██║   ██║       ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║
    ██║   ██║  ██║██║  ██║   ██║       ██║  ██║╚██████╔╝╚██████╔╝   ██║       ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝
    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝       ╚═╝  ╚═╝ ╚═════╝  ╚═════╝    ╚═╝       ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝
*/

pub trait RootMenu {
    fn update_menu(self: &Self, menu: &TrayMenu);
    fn remove_unused_menu_items(self: &Self, cache_key: &str, touched_keys: &[String]);
    fn remove_unused_items(self: &Self, cache_key: &str, touched_keys: &[String]);
    fn add_menu_items(self: &Self, menu: Arc<Box>, items: Vec<MenuItem>) -> Arc<Box>;
    fn add_submenu_items(self: &Self, id: i32, menu: Arc<Box>, items: Vec<MenuItem>) -> Arc<Box>;
}

impl RootMenu for TrayIcon {
    fn update_menu(self: &Self, menu: &TrayMenu) {
        touched_keys().clear();
        touch_or_init_cached_box(
            &format!("{}/{}", self.address, self.menu_path),
            self.menu_path.as_str(),
            |rows| self.popover.set_child(Some(rows.as_ref())),
            move |rows| self.add_menu_items(rows, menu.submenus.clone()),
        );
        // mop up any items that were removed
        let cache_key = format!("{}/", self.address);
        let tkeys = touched_keys();
        self.remove_unused_menu_items(&cache_key, &tkeys);
        self.remove_unused_items(&cache_key, &tkeys);
    }

    fn remove_unused_menu_items(self: &Self, cache_key: &str, touched_keys: &[String]) {
        let ekeys = Vec::from_iter(
            tray_menu_widget()
                .keys()
                .filter(|key| key.starts_with(cache_key)),
        );
        for key in ekeys {
            if !touched_keys.contains(&key) {
                eprintln!("[{}]: {}({})", "tray".green(), "remove_menu".red(), key);
                let item = tray_menu_widget().get(&key.clone()).unwrap();
                item.unparent();
                tray_menu_widget().remove(&key.clone());
            }
        }
    }

    fn remove_unused_items(self: &Self, cache_key: &str, touched_keys: &[String]) {
        let ekeys = Vec::from_iter(menu_item().keys().filter(|key| key.starts_with(cache_key)));
        for key in ekeys {
            if !touched_keys.contains(&key) {
                eprintln!("[{}]: {}({})", "tray".green(), "remove_item".red(), key);
                let item = menu_item().get(&key.clone()).unwrap();
                unsafe {
                    let item: &mut TrayMenuItem = &mut *Arc::clone(&item).as_ptr();
                    item.button.unparent();
                }
                menu_item().remove(&key.clone());
            }
        }
    }

    fn add_menu_items(self: &Self, menu: Arc<Box>, items: Vec<MenuItem>) -> Arc<Box> {
        let cache_key = &format!("{}/menu/0/section", self.address);
        self.add_sections(cache_key, menu, items)
    }

    fn add_submenu_items(self: &Self, id: i32, menu: Arc<Box>, items: Vec<MenuItem>) -> Arc<Box> {
        let cache_key = &format!("{}/menu/0/submenu/{}/section", self.address, id);
        self.add_sections(cache_key, menu, items)
    }
}
