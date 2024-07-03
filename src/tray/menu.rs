use super::{
    menu_box,
    menu_item::{menu_item, MenuItem as TrayMenuItem},
    section::*,
    touch_or_init_cached_box, touched_keys,
    tray_icon::*,
};
use colored::Colorize;
use gtk4::{prelude::*, Box};
use regex::Regex;
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
    fn remove_unused_menus(self: &Self, cache_key: &str, touched_keys: &[String]);
    fn remove_unused_items(self: &Self, cache_key: &str, touched_keys: &[String]);
    fn add_menu_items(self: &Self, menu: Arc<Box>, items: Vec<MenuItem>) -> Arc<Box>;
    fn add_submenu_items(self: &Self, id: i32, menu: Arc<Box>, items: Vec<MenuItem>) -> Arc<Box>;
}

impl RootMenu for TrayIcon {
    fn update_menu(self: &Self, menu: &TrayMenu) {
        touched_keys().clear();
        touch_or_init_cached_box(
            &format!("{}/menu/0", self.address),
            &self.menu_path,
            |rows, cache_key| {
                self.stack
                    .add_named(rows.as_ref(), Some(cache_key.as_str()));
                self.stack.set_visible_child_name(cache_key);
            },
            move |rows, _| self.add_menu_items(rows, menu.submenus.clone()),
        );
        // mop up any items that were removed
        let cache_key = format!("{}/", self.address);
        let tkeys = touched_keys();
        self.remove_unused_menus(&cache_key, &tkeys);
        self.remove_unused_items(&cache_key, &tkeys);
    }

    fn remove_unused_menus(self: &Self, cache_key: &str, touched_keys: &[String]) {
        let ekeys = Vec::from_iter(menu_box().keys().filter(|key| key.starts_with(cache_key)));
        let regex_is_submenu = Regex::new(r"submenu/[^/]+$").unwrap();
        for key in ekeys {
            if !touched_keys.contains(&key) {
                eprintln!("[{}]: {}({})", "tray".green(), "remove_menu".red(), key);
                let item = Arc::clone(menu_box().get(&key.clone()).unwrap());
                if self.stack.child_by_name(key.as_str()).is_some() {
                    self.stack.remove(item.as_ref());
                }
                if item.parent().is_some() {
                    item.unparent();
                }
                menu_box().remove(&key.clone());
                if regex_is_submenu.is_match(&key) {
                    let key = &format!("{}/menu/0", self.address);
                    eprintln!("[{}]: {}({})", "tray".green(), "show_menu".yellow(), key);
                    self.stack.set_visible_child_name(key);
                }
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
                    if item.button.parent().is_some() {
                        item.button.unparent();
                    }
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
