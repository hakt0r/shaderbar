use colored::Colorize;
use gtk4::{
    gdk::{
        gdk_pixbuf::{Colorspace, Pixbuf},
        glib::Bytes,
        prelude::*,
    },
    prelude::*,
    Image, MenuButton, Popover, Stack, StackTransitionType,
};
use std::sync::Arc;
use system_tray::item::StatusNotifierItem;

use crate::utils::early_return;

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
    pub image: Image,
    pub popover: Arc<Popover>,
    pub stack: Arc<Stack>,
    pub status_notifier_item: StatusNotifierItem,
    pub address: String,
    pub menu_path: String,
}

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
        let icon_name = status_notifier_item
            .icon_name
            .as_ref()
            .unwrap_or(&status_notifier_item.id);
        let popover = Arc::new(Popover::builder().build());
        let button = MenuButton::builder().css_classes(["tray"]).build();
        button.set_popover(Some(popover.as_ref()));
        let image = match status_notifier_item.icon_pixmap.as_ref() {
            Some(icon_pixbuf) => {
                let pixbuf = icon_pixbuf[0].clone();
                let pixels = Bytes::from_owned(pixbuf.pixels.to_vec());
                let width = pixbuf.width;
                let height = pixbuf.height;
                let texture = gtk4::gdk::Texture::for_pixbuf(&Pixbuf::from_bytes(
                    &pixels,
                    Colorspace::Rgb,
                    true,
                    8,
                    width,
                    height,
                    4 * width,
                ));
                Image::from_paintable(Some(&texture))
            }
            None => Image::from_icon_name(icon_name),
        };
        image.add_css_class("icon");
        let row = gtk4::CenterBox::builder().build();
        row.set_center_widget(Some(&image));
        button.set_child(Some(&row));
        button.set_width_request(24);
        let stack = Stack::builder()
            .transition_type(StackTransitionType::SlideLeftRight)
            .transition_duration(100)
            .build();
        popover.set_child(Some(&stack));
        let trigger = gtk4::ShortcutTrigger::parse_string("F1");
        let manager = gtk4::ShortcutController::new();
        manager.set_scope(gtk4::ShortcutScope::Global);
        let shortcut = gtk4::Shortcut::builder().build();
        let action = gtk4::ShortcutAction::parse_string("activate");
        shortcut.set_action(action);
        shortcut.set_trigger(trigger);
        shortcut.set_arguments(Some(&menu_path.to_variant()));
        shortcut.connect_action_notify(move |action| {
            let menu_path = action.arguments().unwrap().get::<String>().unwrap();
            eprintln!(
                "[{}]: {}({})",
                "tray".green(),
                "shortcut".yellow(),
                menu_path
            );
        });
        manager.add_shortcut(shortcut);
        button.add_controller(manager);
        Self {
            button,
            image,
            popover,
            stack: Arc::new(stack),
            status_notifier_item: status_notifier_item.clone(),
            address: id.clone(),
            menu_path,
        }
    }

    pub fn set_icon(self: &Self, icon: Option<String>) {
        eprintln!(
            "[{}]: {}({:?})",
            "tray_icon".green(),
            "set_icon".yellow(),
            icon.clone(),
        );
        early_return!(icon.is_none());
        let icon = icon.unwrap();
        self.image.set_icon_name(Some(icon.as_str()));
    }

    pub fn set_title(self: &Self, title: Option<String>) {
        eprintln!(
            "[{}]: {}({:?})",
            "tray_icon".green(),
            "set_title".yellow(),
            title.clone(),
        );
    }
    pub fn set_status(self: &Self, status: system_tray::item::Status) {
        eprintln!(
            "[{}]: {}({:?})",
            "tray_icon".green(),
            "set_status".yellow(),
            status,
        );
    }
}
