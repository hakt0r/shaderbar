use colored::Colorize;
use gtk4::{prelude::*, MenuButton, Popover, Stack, StackTransitionType};
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
        let button = MenuButton::builder().css_classes(["tray"]).build();
        let default_icon = &"volume-up".to_string();
        let icon_name = status_notifier_item
            .icon_name
            .as_ref()
            .unwrap_or(default_icon);
        button.set_icon_name(icon_name.as_str());
        let popover = Arc::new(Popover::builder().build());
        button.set_popover(Some(popover.as_ref()));
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
            popover,
            stack: Arc::new(stack),
            status_notifier_item: status_notifier_item.clone(),
            address: id.clone(),
            menu_path,
        }
    }
}
