mod menu;
mod menu_item;
mod section;
mod submenu;
mod tray_icon;

use crate::utils::{global, global_init};
use colored::Colorize;
use glib::spawn_future_local;
use gtk4::{glib, prelude::*, Box};
use menu::RootMenu;
use std::{borrow::BorrowMut, collections::HashMap, sync::Arc};
use system_tray::client::UpdateEvent::*;
use system_tray::client::{Client, Event::*};
use tray_icon::TrayIcon;

macro_rules! log {
    ($call:expr, $arg:expr) => {
        eprintln!("[{}]: {}({})", "tray".green(), $call.yellow(), $arg);
    };
}

pub struct Tray {
    pub client: Option<Client>,
    pub items: HashMap<String, TrayIcon>,
    pub widget: gtk4::Box,
}

global_init!(tray, Tray, init_tray);

fn init_tray() -> Tray {
    spawn_future_local(init_tray_async());
    return Tray {
        client: None,
        items: HashMap::new(),
        widget: gtk4::Box::builder()
            .spacing(0)
            .orientation(gtk4::Orientation::Horizontal)
            .build(),
    };
}

async fn init_tray_async() {
    let pid: u32 = std::process::id();
    let process_name = "shaderbar";
    let process_id = format!("{}-{}", process_name, pid);
    let client: Client = Client::new(&process_id).await.unwrap();
    let rx = &mut client.subscribe();
    let tray = tray();
    let widget = tray.widget.clone();
    tray.client = Some(client);

    while let Ok(ev) = rx.recv().await {
        let items = tray.items.borrow_mut();
        match ev {
            Add(id, item) => {
                let existing_item = items.get(&id);
                if existing_item.is_some() {
                    log!("item_exists", id);
                    continue;
                }
                let item = TrayIcon::new(&id, item.as_ref());
                widget.append(&item.button);
                items.insert(id, item);
            }
            Update(id, item) => {
                let tray_item = items.get(&id);
                if tray_item.is_none() {
                    log!("UnknownTrayIcon", id);
                    continue;
                }
                let tray_item = tray_item.unwrap();
                match item {
                    Menu(menu) => tray_item.update_menu(&menu),
                    Icon(icon) => tray_item.set_icon(icon),
                    OverlayIcon(icon) => tray_item.set_icon(icon),
                    AttentionIcon(icon) => tray_item.set_icon(icon),
                    Title(title) => tray_item.set_title(title),
                    Status(status) => tray_item.set_status(status),
                }
            }
            Remove(id) => {
                log!("remove_item", id);
                let item = items.get(&id);
                if item.is_none() {
                    log!("NotFound", id);
                    continue;
                }
                let item = item.unwrap();
                widget.remove(&item.button);
                items.remove(&id);
            }
        }
    }
}

/*
██████╗  ██████╗ ██╗  ██╗███████╗███████╗
██╔══██╗██╔═══██╗╚██╗██╔╝██╔════╝██╔════╝
██████╔╝██║   ██║ ╚███╔╝ █████╗  ███████╗
██╔══██╗██║   ██║ ██╔██╗ ██╔══╝  ╚════██║
██████╔╝╚██████╔╝██╔╝ ██╗███████╗███████║
╚═════╝  ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝
*/

global!(menu_box, HashMap<String, Arc<Box>>, HashMap::new());
global!(touched_keys, Vec<String>, Vec::new());

pub fn cached_box(
    cache_key: &String,
    alias: &String,
    init: impl FnOnce(Arc<Box>, &String),
    touch: impl FnOnce(Arc<Box>, &String) -> Arc<Box>,
) {
    log!("cached_box", format!("{}, {}", cache_key, alias));
    touched_keys().push(cache_key.to_string());
    let widget = match menu_box().get(cache_key) {
        Some(widget) => Arc::clone(widget),
        None => {
            let rows = Arc::new(
                Box::builder()
                    .orientation(gtk4::Orientation::Vertical)
                    .build(),
            );
            init(Arc::clone(&rows), &cache_key);
            menu_box().insert(cache_key.clone(), rows);
            Arc::clone(menu_box().get(cache_key).unwrap())
        }
    };
    touch(Arc::clone(&widget), &cache_key);
}
