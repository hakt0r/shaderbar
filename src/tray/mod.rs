mod diff;
mod icon;
mod interface;

use crate::tray::diff::get_diffs;
use color_eyre::{Report, Result};
use gtk4::IconTheme;
use interface::TrayMenu;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, OnceLock};
use system_tray::client::Event;
use system_tray::client::{ActivateRequest, UpdateEvent};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};

#[must_use]
pub fn runtime() -> Arc<Runtime> {
    static RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();
    RUNTIME.get_or_init(|| Arc::new(create_runtime())).clone()
}

fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio to create a valid runtime")
}

pub fn spawn<F>(f: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    runtime().spawn(f)
}

#[macro_export]
macro_rules! send_async {
    ($tx:expr, $msg:expr) => {
        $tx.send($msg).await.expect(crate::ERR_CHANNEL_SEND)
    };
}

#[macro_export]
macro_rules! try_send {
    ($tx:expr, $msg:expr) => {
        $tx.try_send($msg).expect(crate::ERR_CHANNEL_SEND)
    };
}

#[macro_export]
macro_rules! glib_recv {
    ($rx:expr, $val:ident => $expr:expr) => {{
        glib::spawn_future_local(async move {
            // re-delcare in case ie `context.subscribe()` is passed directly
            let mut rx = $rx;
            loop {
                match rx.recv().await {
                    Ok($val) => $expr,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(count)) => {
                        tracing::warn!("Channel lagged behind by {count}, this may result in unexpected or broken behaviour");
                    }
                    Err(err) => {
                        tracing::error!("{err:?}");
                        break;
                    }
                }
            }
        });
    }};
}

const fn default_icon_size() -> u32 {
    16
}

type ReceiveMessage = Event;

fn spawn_controller(
    widget: gtk4::Widget,
    context: &WidgetContext<Self::SendMessage, Self::ReceiveMessage>,
    mut rx: mpsc::Receiver<ReceiveMessage>,
) -> Result<()> {
    let tx = context.tx.clone();

    let client = context.try_client::<tray::Client>()?;
    let mut tray_rx = client.subscribe();

    let initial_items = lock!(client.items()).clone();
    spawn(async move {
        for (key, (item, menu)) in initial_items {
            send_async!(
                tx,
                ModuleUpdateEvent::Update(Event::Add(key.clone(), item.into()))
            );

            if let Some(menu) = menu.clone() {
                send_async!(
                    tx,
                    ModuleUpdateEvent::Update(Event::Update(key, UpdateEvent::Menu(menu)))
                );
            }
        }

        while let Ok(message) = tray_rx.recv().await {
            send_async!(tx, ModuleUpdateEvent::Update(message));
        }
    });

    // send tray commands
    spawn(async move {
        while let Some(cmd) = rx.recv().await {
            client.activate(cmd).await?;
        }

        Ok::<_, Report>(())
    });

    Ok(())
}

fn into_widget(widget: gtk4::Widget) {
    let container = MenuBar::new();
    let direction = PackDirection::Ltr;

    container.set_pack_direction(direction);
    container.set_child_pack_direction(direction);

    {
        let container = container.clone();
        let mut menus = HashMap::new();
        let icon_theme = info.icon_theme.clone();

        // listen for UI updates
        glib_recv!(context.subscribe(), update =>
            on_update(update, &container, &mut menus, &icon_theme, self.icon_size, self.prefer_theme_icons, &context.controller_tx)
        );
    };
}

/// Handles UI updates as callback,
/// getting the diff since the previous update and applying it to the menu.
fn on_update(
    update: Event,
    container: &MenuBar,
    menus: &mut HashMap<Box<str>, TrayMenu>,
    icon_theme: &IconTheme,
    icon_size: u32,
    prefer_icons: bool,
    tx: &mpsc::Sender<ActivateRequest>,
) {
    match update {
        Event::Add(address, item) => {
            debug!("Received new tray item at '{address}': {item:?}");

            let mut menu_item = TrayMenu::new(tx.clone(), address.clone(), *item);
            container.add(&menu_item.widget);

            if let Ok(image) = icon::get_image(&menu_item, icon_theme, icon_size, prefer_icons) {
                menu_item.set_image(&image);
            } else {
                let label = menu_item.title.clone().unwrap_or(address.clone());
                menu_item.set_label(&label);
            };

            menu_item.widget.show();
            menus.insert(address.into(), menu_item);
        }
        Event::Update(address, update) => {
            debug!("Received tray update for '{address}': {update:?}");

            let Some(menu_item) = menus.get_mut(address.as_str()) else {
                error!("Attempted to update menu at '{address}' but could not find it");
                return;
            };

            match update {
                UpdateEvent::AttentionIcon(_icon) => {
                    warn!("received unimplemented NewAttentionIcon event");
                }
                UpdateEvent::Icon(icon) => {
                    if icon.as_ref() != menu_item.icon_name() {
                        match icon::get_image(menu_item, icon_theme, icon_size, prefer_icons) {
                            Ok(image) => menu_item.set_image(&image),
                            Err(_) => menu_item.show_label(),
                        };
                    }

                    menu_item.set_icon_name(icon);
                }
                UpdateEvent::OverlayIcon(_icon) => {
                    warn!("received unimplemented NewOverlayIcon event");
                }
                UpdateEvent::Status(_status) => {
                    warn!("received unimplemented NewStatus event");
                }
                UpdateEvent::Title(title) => {
                    if let Some(label_widget) = menu_item.label_widget() {
                        label_widget.set_label(&title.unwrap_or_default());
                    }
                }
                // UpdateEvent::Tooltip(_tooltip) => {
                //     warn!("received unimplemented NewAttentionIcon event");
                // }
                UpdateEvent::Menu(menu) => {
                    debug!("received new menu for '{}'", address);

                    let diffs = get_diffs(menu_item.state(), &menu.submenus);

                    menu_item.apply_diffs(diffs);
                    menu_item.set_state(menu.submenus);
                }
            }
        }
        Event::Remove(address) => {
            debug!("Removing tray item at '{address}'");

            if let Some(menu) = menus.get(address.as_str()) {
                container.remove(&menu.widget);
            }
        }
    };
}
