use glib::spawn_future_local;
use std::{borrow::BorrowMut, collections::HashMap};
use system_tray::{client::Client, item::StatusNotifierItem, menu::TrayMenu};
mod diff;
pub mod icon;
use gtk4::glib;

// use diff::get_diffs;

pub async fn create_tray() {
    //spawn_future_local(inite_deno_server());
    spawn_future_local(async move {
        let pid: u32 = std::process::id();
        let process_name = "shaderbar";
        let process_id = format!("{}-{}", process_name, pid);
        eprintln!("Creating tray for process: {}", process_id);
        let client: Client = Client::new(&process_id).await.unwrap();
        let rx = &mut client.subscribe();
        let initial_items = client.items();
        let iterabe_items = initial_items.lock().unwrap();
        let tray = tray();
        tray.client = Some(client);
        drop(iterabe_items);

        while let Ok(ev) = rx.recv().await {
            // get_diffs(&ev.old, &ev).iter().for_each(|diff| {
            //     eprintln!("{diff:?}");
            // });
            // println!("{ev:?}");
            let items = tray.items.borrow_mut();
            match ev {
                system_tray::client::Event::Add(id, item) => {
                    icon::add(id.clone(), item.icon_name.unwrap());
                    items.insert(
                        id.clone(),
                        TrayItem {
                            item: None,
                            menu: None,
                        },
                    );
                }
                system_tray::client::Event::Update(id, item) => {
                    eprintln!("Updating tray item: {}\n{:?}", id, item);
                }
                system_tray::client::Event::Remove(id) => {
                    icon::remove(id.clone());
                    items.remove(&id);
                }
            }
        }
    });
}

pub struct Tray {
    pub client: Option<Client>,
    pub items: HashMap<String, TrayItem>,
}

pub struct TrayItem {
    pub item: Option<StatusNotifierItem>,
    pub menu: Option<TrayMenu>,
}

pub fn tray() -> &'static mut Tray {
    static mut TRAY: Option<Tray> = None;
    unsafe {
        return TRAY.get_or_insert_with(init_tray);
    }
}

fn init_tray() -> Tray {
    return Tray {
        client: None,
        items: HashMap::new(),
    };
}
