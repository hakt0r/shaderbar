use super::menu_item::MenuItem as TrayMenuItem;
use crate::utils::global;
use colored::Colorize;
use gtk4::{Box, MenuButton, Popover};
use std::{cell::Cell, collections::HashMap, sync::Arc};
use system_tray::{
    item::StatusNotifierItem,
    menu::{ToggleState, ToggleType},
};

/*
 ████████╗██████╗  █████╗ ██╗   ██╗    ██╗ ██████╗ ██████╗ ███╗   ██╗
 ╚══██╔══╝██╔══██╗██╔══██╗╚██╗ ██╔╝    ██║██╔════╝██╔═══██╗████╗  ██║
    ██║   ██████╔╝███████║ ╚████╔╝     ██║██║     ██║   ██║██╔██╗ ██║
    ██║   ██╔══██╗██╔══██║  ╚██╔╝      ██║██║     ██║   ██║██║╚██╗██║
    ██║   ██║  ██║██║  ██║   ██║       ██║╚██████╗╚██████╔╝██║ ╚████║
    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝       ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝
*/

pub struct TrayIcon {
    pub button: MenuButton,
    pub popover: Arc<Popover>,
    pub status_notifier_item: StatusNotifierItem,
    pub address: String,
    pub menu_path: String,
}

global!(touched_keys, Vec<String>, Vec::new());

/*
██████╗  ██████╗ ██╗  ██╗███████╗███████╗
██╔══██╗██╔═══██╗╚██╗██╔╝██╔════╝██╔════╝
██████╔╝██║   ██║ ╚███╔╝ █████╗  ███████╗
██╔══██╗██║   ██║ ██╔██╗ ██╔══╝  ╚════██║
██████╔╝╚██████╔╝██╔╝ ██╗███████╗███████║
╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝
*/

global!(tray_menu_widget, HashMap<String, Arc<Box>>, HashMap::new());

pub fn cached_box(id: &String) -> (Arc<Box>, bool) {
    match tray_menu_widget().get(id) {
        Some(widget) => (Arc::clone(widget), true),
        None => {
            let rows = Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .build();
            tray_menu_widget().insert(id.clone(), Arc::new(rows));
            (Arc::clone(tray_menu_widget().get(id).unwrap()), false)
        }
    }
}

pub fn touch_cached_box(cache_key: &str, alias: &str) -> (Arc<Box>, bool) {
    eprintln!(
        "[{}]: {}({}) @{}",
        "tray".green(),
        "cached_box".yellow(),
        cache_key.blue(),
        alias.magenta()
    );
    touched_keys().push(cache_key.to_string());
    cached_box(&cache_key.to_string())
}

pub fn touch_or_init_cached_box(
    cache_key: &str,
    alias: &str,
    init: impl FnOnce(Arc<Box>),
    touch: impl FnOnce(Arc<Box>) -> Arc<Box>,
) {
    let (widget, was_cached) = touch_cached_box(cache_key, alias);
    touch(Arc::clone(&widget));
    if !was_cached {
        init(Arc::clone(&widget));
    }
}

/*
 ████████╗██████╗  █████╗ ██╗   ██╗    ██╗████████╗███████╗███╗   ███╗
 ╚══██╔══╝██╔══██╗██╔══██╗╚██╗ ██╔╝    ██║╚══██╔══╝██╔════╝████╗ ████║
    ██║   ██████╔╝███████║ ╚████╔╝     ██║   ██║   █████╗  ██╔████╔██║
    ██║   ██╔══██╗██╔══██║  ╚██╔╝      ██║   ██║   ██╔══╝  ██║╚██╔╝██║
    ██║   ██║  ██║██║  ██║   ██║       ██║   ██║   ███████╗██║ ╚═╝ ██║
    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝       ╚═╝   ╚═╝   ╚══════╝╚═╝     ╚═╝
*/

impl TrayIcon {
    pub fn new(id: &String, status_notifier_item: &StatusNotifierItem) -> Self {
        let menu_path = String::clone(status_notifier_item.menu.as_ref().unwrap());
        eprintln!(
            "[{}]: {}({},{})",
            "tray".green(),
            "add_item".yellow(),
            id,
            status_notifier_item.id
        );
        let button = MenuButton::builder().css_classes(["tray"]).build();
        let default_icon = &"volume-up".to_string();
        let icon_name = status_notifier_item
            .icon_name
            .as_ref()
            .unwrap_or(default_icon);
        button.set_icon_name(icon_name.as_str());
        let popover = Arc::new(Popover::builder().build());
        button.set_popover(Some(popover.as_ref()));
        Self {
            button,
            popover,
            status_notifier_item: status_notifier_item.clone(),
            address: id.clone(),
            menu_path,
        }
    }
}

/*
 ███╗   ███╗███████╗███╗   ██╗██╗   ██╗███████╗
 ████╗ ████║██╔════╝████╗  ██║██║   ██║██╔════╝
 ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║███████╗
 ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║╚════██║
 ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝███████║
 ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
*/

global!(
    menu_item,
    HashMap<String, Arc<Cell<TrayMenuItem>>>,
    HashMap::new()
);

pub fn touch_or_init_menu(
    cache_key: &str,
    label: Option<String>,
    enabled: bool,
    icon_name: Option<String>,
    toggle_type: ToggleType,
    toggle_state: ToggleState,
    has_submenu: bool,
) -> (Arc<Cell<TrayMenuItem>>, bool) {
    touched_keys().push(cache_key.to_string());
    let cached_item = menu_item().get(cache_key);
    let label_clone = label.unwrap().clone();
    let (item, was_cached) = match cached_item {
        Some(item) => (Arc::clone(item), true),
        None => {
            menu_item().insert(
                cache_key.to_string(),
                Arc::new(Cell::new(TrayMenuItem::new(
                    cache_key,
                    enabled,
                    label_clone.as_str(),
                    icon_name,
                    toggle_type,
                    toggle_state,
                    has_submenu,
                ))),
            );
            eprintln!(
                "[{}]: {}({}) @{}",
                "tray".green(),
                "add_menu_item".yellow(),
                cache_key,
                label_clone
            );
            (Arc::clone(menu_item().get(cache_key).unwrap()), false)
        }
    };
    (item, was_cached)
}