use colored::Colorize;
use gtk4::{MenuButton, Popover};
use std::sync::Arc;
use system_tray::item::StatusNotifierItem;

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
