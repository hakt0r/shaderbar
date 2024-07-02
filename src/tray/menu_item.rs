/*
 ███╗   ███╗███████╗███╗   ██╗██╗   ██╗    ██╗    ██╗██╗██████╗  ██████╗ ███████╗████████╗
 ████╗ ████║██╔════╝████╗  ██║██║   ██║    ██║    ██║██║██╔══██╗██╔════╝ ██╔════╝╚══██╔══╝
 ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║    ██║ █╗ ██║██║██║  ██║██║  ███╗█████╗     ██║
 ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║    ██║███╗██║██║██║  ██║██║   ██║██╔══╝     ██║
 ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝    ╚███╔███╔╝██║██████╔╝╚██████╔╝███████╗   ██║
 ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝      ╚══╝╚══╝ ╚═╝╚═════╝  ╚═════╝ ╚══════╝   ╚═╝
*/

use super::{
    menu::touch_or_init_menu, menu_item::MenuItem as TrayMenuItem, submenu::Submenu, tray,
    tray_icon::TrayIcon,
};
use crate::utils::global;
use colored::Colorize;
use glib::spawn_future_local;
use gtk4::{
    glib, prelude::*, Box, Button, CheckButton, Image, Label, Orientation::Horizontal, Widget,
};
use std::sync::Arc;
use system_tray::{
    client::ActivateRequest,
    menu::{
        MenuItem as SystrayMenuItem,
        ToggleState::{self, Indeterminate, Off, On},
        ToggleType::{self, CannotBeToggled, Checkmark, Radio},
    },
};

global!(
    blacklist,
    Vec<&'static str>,
    vec!["bluetooth-symbolic", "bluetooth-disabled-symbolic"]
);

pub struct MenuItem {
    pub id: String,
    enabled: bool,
    pub button: Button,
    pub label: Label,
    pub label_text: String,
    pub toggle_type: ToggleType,
    pub toggle_state: ToggleState,
    pub prefix: Widget,
}

pub trait MenuItems {
    fn add_menu_item(self: &Self, menu: &Arc<Box>, child: SystrayMenuItem);
    fn connect_activate(self: &Self, id: i32, menu_item_button: &gtk4::Button);
}

impl MenuItems for TrayIcon {
    fn add_menu_item(self: &Self, menu: &Arc<Box>, child: SystrayMenuItem) {
        let SystrayMenuItem {
            id,
            label,
            enabled,
            toggle_type,
            toggle_state,
            icon_name,
            submenu,
            ..
        } = child;
        if label.is_none() {
            return;
        }
        let has_submenu = !submenu.is_empty();
        let label = label.unwrap();

        let (item, was_cached) = touch_or_init_menu(
            &format!("{}/item/{}", self.address, id),
            Some(label.clone()),
            enabled,
            icon_name,
            toggle_type,
            toggle_state,
            has_submenu,
        );

        unsafe {
            let item: &mut TrayMenuItem = &mut *Arc::clone(&item).as_ptr();
            let label_str = label.as_str();
            item.apply_state(enabled, label_str, toggle_type, toggle_state);
            if !has_submenu {
                self.connect_activate(id, &item.button);
            } else {
                self.connect_submenu_activate(id, &item.button);
                self.add_submenu(id, label, submenu.clone());
            }
            if !was_cached {
                menu.append(&item.button);
            }
        }
    }

    fn connect_activate(self: &Self, id: i32, menu_item_button: &gtk4::Button) {
        let menu_path = format!("{}", self.menu_path);
        let address = format!("{}", self.address);
        menu_item_button.connect_clicked(move |_| {
            eprintln!(
                "[{}]: {}({},{})",
                "tray".green(),
                "activate".yellow(),
                menu_path,
                id
            );
            spawn_future_local(tray().client.as_mut().unwrap().activate(ActivateRequest {
                address: address.clone(),
                menu_path: menu_path.clone(),
                submenu_id: id.clone(),
            }));
        });
    }
}

impl MenuItem {
    pub fn new(
        id: &str,
        enabled: bool,
        label_text: &str,
        icon_name: Option<String>,
        toggle_type: ToggleType,
        toggle_state: ToggleState,
        has_submenu: bool,
    ) -> MenuItem {
        let row = Box::builder().orientation(Horizontal).spacing(8).build();

        let prefix: Widget = match (has_submenu, toggle_type) {
            (true, _) => Image::from_icon_name("open-menu").into(),
            (_, ToggleType::Checkmark) => CheckButton::builder().build().into(),
            (_, ToggleType::Radio) => CheckButton::builder().build().into(),
            (_, ToggleType::CannotBeToggled) => match icon_name {
                None => Image::new().into(),
                Some(icon) => match blacklist().contains(&icon.as_str()) {
                    true => Image::builder().build().into(),
                    false => {
                        eprintln!("Icon: {:?}({:?}", icon, label_text);
                        let icon = Image::from_icon_name(icon.as_str());
                        eprintln!("Icon: {:?}", icon);
                        icon.into()
                    }
                },
            },
        };

        let label = Label::builder().label(label_text).build();
        row.append(&prefix);
        row.append(&label);

        let button = Button::builder().build();
        button.add_css_class("tray-menu-item");
        button.set_child(Some(&row));

        return MenuItem {
            id: id.to_string(),
            enabled,
            button,
            label,
            label_text: label_text.to_string(),
            toggle_type,
            toggle_state,
            prefix,
        };
    }

    pub fn apply_state(
        self: &mut Self,
        enabled: bool,
        label_text: &str,
        toggle_type: ToggleType,
        toggle_state: ToggleState,
    ) {
        self.enabled = enabled;
        self.toggle_type = toggle_type;
        self.toggle_state = toggle_state;
        self.label_text = label_text.to_string();
        self.label.set_text(label_text);
        match toggle_type {
            Checkmark | Radio => {
                let check_button = self.prefix.clone().downcast::<CheckButton>().unwrap();
                match toggle_state {
                    On => check_button.set_active(true),
                    Off => check_button.set_active(false),
                    Indeterminate => check_button.set_inconsistent(true),
                }
            }
            CannotBeToggled => {}
        }

        if self.enabled {
            // eprintln!("Enabled: {:?}({:?})", self.id, self.label_text);
            self.button.remove_css_class("disabled");
            self.button.set_sensitive(true);
            self.label.set_sensitive(true);
            self.prefix.set_sensitive(true);
        } else {
            // eprintln!("Disabled: {:?}({:?})", self.id, self.label_text);
            self.button.add_css_class("disabled");
            self.button.set_sensitive(false);
            self.label.set_sensitive(false);
            self.prefix.set_sensitive(false);
        }
    }
}

impl std::fmt::Debug for MenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuItem")
            .field("id", &self.id)
            .field("enabled", &self.enabled)
            .field("label_text", &self.label_text)
            .field("toggle_type", &self.toggle_type)
            .field("toggle_state", &self.toggle_state)
            .finish()
    }
}
