use glib::spawn_future_local;
use gtk4::{
    gio::{self, ActionEntry, SimpleActionGroup},
    glib,
    prelude::*,
    MenuButton,
};
use std::{borrow::BorrowMut, collections::HashMap};
use system_tray::{
    client::{ActivateRequest, Client},
    item::StatusNotifierItem,
    menu::{MenuItem, ToggleState, ToggleType},
};
pub mod diff;

pub struct Tray {
    pub client: Option<Client>,
    pub items: HashMap<String, TrayItem>,
    pub widget: gtk4::Box,
}

pub struct TrayItem {
    pub item: Option<StatusNotifierItem>,
    pub icon: MenuButton,
}

crate::utils::global_init!(tray, Tray, init_tray);
fn init_tray() -> Tray {
    return Tray {
        client: None,
        items: HashMap::new(),
        widget: gtk4::Box::builder()
            .spacing(0)
            .orientation(gtk4::Orientation::Horizontal)
            .build(),
    };
}

pub async fn create_tray() {
    spawn_future_local(async move {
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
                system_tray::client::Event::Add(id, item) => {
                    let id_clone = id.clone();
                    let btn = MenuButton::builder().css_classes(["tray"]).build();
                    let icon_name = item.icon_name.as_ref().unwrap();
                    btn.set_icon_name(icon_name.as_str());
                    widget.append(&btn);
                    eprintln!("Adding tray item: {}\n{:?}", id, item);
                    items.insert(
                        id_clone,
                        TrayItem {
                            item: Some(*item),
                            icon: btn,
                        },
                    );
                }
                system_tray::client::Event::Update(id, item) => {
                    //eprintln!("Updating tray item: {}\n{:?}", id, item);
                    match item {
                        system_tray::client::UpdateEvent::Menu(menu) => {
                            let status_notifier_item = items.get(&id);
                            if status_notifier_item.is_none() {
                                continue;
                            }
                            let item = status_notifier_item.unwrap();
                            let actions = SimpleActionGroup::new();
                            let menu = add_menu_items(
                                &format!("{}", id),
                                &format!("{}", item.item.as_ref().unwrap().menu.as_ref().unwrap()),
                                gio::Menu::new(),
                                menu.submenus,
                                &actions,
                            );
                            let popover = gtk4::PopoverMenu::from_model(Some(&menu));
                            let icon = &item.icon;
                            icon.set_popover(Some(&popover));
                            popover.insert_action_group("tray", Some(&actions))
                        }
                        _ => {
                            eprintln!("Unhandled update event: {:?}", item);
                        }
                    }
                }
                system_tray::client::Event::Remove(id) => {
                    eprintln!("Removing tray item: {}", id);
                    let item = items.get(&id);
                    if item.is_none() {
                        eprintln!("Item not found: {}", id);
                        continue;
                    }
                    let item = item.unwrap();
                    widget.remove(&item.icon);
                    let icon = widget.first_child().unwrap();
                    icon.unrealize();
                    widget.unrealize();
                    items.remove(&id);
                }
            }
        }
    });
}

/*
 ███╗   ███╗███████╗███╗   ██╗██╗   ██╗    ██╗████████╗███████╗███╗   ███╗███████╗
 ████╗ ████║██╔════╝████╗  ██║██║   ██║    ██║╚══██╔══╝██╔════╝████╗ ████║██╔════╝
 ██╔████╔██║█████╗  ██╔██╗ ██║██║   ██║    ██║   ██║   █████╗  ██╔████╔██║███████╗
 ██║╚██╔╝██║██╔══╝  ██║╚██╗██║██║   ██║    ██║   ██║   ██╔══╝  ██║╚██╔╝██║╚════██║
 ██║ ╚═╝ ██║███████╗██║ ╚████║╚██████╔╝    ██║   ██║   ███████╗██║ ╚═╝ ██║███████║
 ╚═╝     ╚═╝╚══════╝╚═╝  ╚═══╝ ╚═════╝     ╚═╝   ╚═╝   ╚══════╝╚═╝     ╚═╝╚══════╝
*/

fn add_menu_items(
    address: &String,
    menu_path: &String,
    menu: gio::Menu,
    items: Vec<MenuItem>,
    action_group: &gio::SimpleActionGroup,
) -> gio::Menu {
    for child in items {
        let MenuItem {
            id,
            label: label_text,
            enabled,
            toggle_type,
            toggle_state,
            icon_name,
            ..
        } = child;
        if let Some(label) = label_text {
            if child.submenu.is_empty() {
                let action: Option<&str> = create_action(
                    &address.clone(),
                    &menu_path.clone(),
                    &id,
                    enabled,
                    toggle_type,
                    toggle_state,
                    action_group,
                );
                let item = gio::MenuItem::new(Some(&label), action);
                if let Some(icon) = icon_name {
                    if let Ok(icon) = gtk4::gio::Icon::for_string(&icon) {
                        item.set_icon(&icon);
                    }
                }
                menu.append_item(&item);
            } else {
                let item = add_menu_items(
                    address,
                    menu_path,
                    gio::Menu::new(),
                    child.submenu,
                    action_group,
                );
                menu.append_submenu(Some(&label), &item);
            }
        } else {
            eprintln!("---------------------");
        }
    }
    menu
}

/*
  █████╗  ██████╗████████╗██╗ ██████╗ ███╗   ██╗███████╗
 ██╔══██╗██╔════╝╚══██╔══╝██║██╔═══██╗████╗  ██║██╔════╝
 ███████║██║        ██║   ██║██║   ██║██╔██╗ ██║███████╗
 ██╔══██║██║        ██║   ██║██║   ██║██║╚██╗██║╚════██║
 ██║  ██║╚██████╗   ██║   ██║╚██████╔╝██║ ╚████║███████║
 ╚═╝  ╚═╝ ╚═════╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝
*/

fn stateful_action(
    address: &String,
    menu_path: &String,
    id: &i32,
    action_group: &gio::SimpleActionGroup,
    key: &str,
    state: &str,
) -> Option<&'static str> {
    let menu_id = id.clone();
    let address = address.clone();
    let menu_path = menu_path.clone();
    let action_key = Box::leak(Box::new(format!("{}{}", key, id)));
    let builder = ActionEntry::builder(&action_key)
        .state(state.to_variant())
        .parameter_type(Some(glib::VariantTy::STRING))
        .activate(move |_, _, _| {
            let tray: &mut Tray = tray();
            spawn_future_local(tray.client.as_ref().unwrap().activate(ActivateRequest {
                address: address.clone(),
                menu_path: menu_path.clone(),
                submenu_id: menu_id,
            }));
        })
        .build();
    action_group.add_action_entries([builder]);
    Some(Box::leak(Box::new(format!("tray.{}", action_key))).as_str())
}

fn create_action(
    address: &String,
    menu_path: &String,
    id: &i32,
    enabled: bool,
    toggle_type: ToggleType,
    toggle_state: ToggleState,
    action_group: &gio::SimpleActionGroup,
) -> Option<&'static str> {
    match (enabled, toggle_type, toggle_state) {
        (true, ToggleType::Checkmark, ToggleState::On) => {
            stateful_action(address, menu_path, id, action_group, "toggle_on_", "On")
        }

        (true, ToggleType::Checkmark, ToggleState::Off) => {
            stateful_action(address, menu_path, id, action_group, "toggle_off_", "Off")
        }

        (true, ToggleType::Radio, ToggleState::On) => {
            stateful_action(address, menu_path, id, action_group, "radio_on_", "On")
        }
        (true, ToggleType::Radio, ToggleState::Off) => {
            stateful_action(address, menu_path, id, action_group, "radio_off_", "Off")
        }
        (true, ToggleType::CannotBeToggled, _) => {
            let menu_id = id.clone();
            let address = address.clone();
            let menu_path = menu_path.clone();
            let action_key = Box::leak(Box::new(format!("press_{}", id)));
            action_group.add_action_entries([ActionEntry::builder(&action_key)
                .activate(move |_, _, _| {
                    let tray: &mut Tray = tray();
                    spawn_future_local(tray.client.as_ref().unwrap().activate(ActivateRequest {
                        address: address.clone(),
                        menu_path: menu_path.clone(),
                        submenu_id: menu_id,
                    }));
                })
                .build()]);
            Some(Box::leak(Box::new(format!("tray.{}", action_key))).as_str())
        }
        (false, _, _) => Some("tray.disabled"),
        _ => None,
    }
}
