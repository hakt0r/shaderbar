use super::{cached_box, menu_item::*, tray_icon::TrayIcon};
use gtk4::{prelude::*, Box};
use std::sync::Arc;
use system_tray::menu::MenuItem;

/*
 ███████╗███████╗ ██████╗████████╗██╗ ██████╗ ███╗   ██╗
 ██╔════╝██╔════╝██╔════╝╚══██╔══╝██║██╔═══██╗████╗  ██║
 ███████╗█████╗  ██║        ██║   ██║██║   ██║██╔██╗ ██║
 ╚════██║██╔══╝  ██║        ██║   ██║██║   ██║██║╚██╗██║
 ███████║███████╗╚██████╗   ██║   ██║╚██████╔╝██║ ╚████║
 ╚══════╝╚══════╝ ╚═════╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝
*/

pub trait SectionMenuItems {
    fn add_sections(self: &Self, cache_key: &str, menu: Arc<Box>, items: Vec<MenuItem>)
        -> Arc<Box>;
    fn add_menu_section(self: &Self, menu: Arc<Box>, items: Vec<MenuItem>) -> Arc<Box>;
}

impl SectionMenuItems for TrayIcon {
    fn add_sections(
        self: &Self,
        cache_key: &str,
        menu: Arc<Box>,
        items: Vec<MenuItem>,
    ) -> Arc<Box> {
        let mut sections = 0;
        for section in to_sections(items) {
            let items_for_section = section.clone();
            let cache_key = &format!("{}/{}", cache_key, section[0].id);
            cached_box(
                &cache_key,
                section[0].label.as_ref().unwrap_or(&String::from("<NULL>")),
                |section_menu, _| {
                    section_menu.add_css_class("section");
                    if sections == 0 {
                        section_menu.add_css_class("first");
                    }
                    menu.append(section_menu.as_ref());
                },
                move |section_menu, _| {
                    self.add_menu_section(Arc::clone(&section_menu), items_for_section)
                },
            );
            sections += 1;
        }
        menu
    }

    fn add_menu_section(self: &Self, menu: Arc<Box>, items: Vec<MenuItem>) -> Arc<Box> {
        for child in items {
            self.add_menu_item(&menu, child);
        }
        menu
    }
}

/*
 ████████╗ ██████╗     ███████╗███████╗ ██████╗████████╗██╗ ██████╗ ███╗   ██╗███████╗
 ╚══██╔══╝██╔═══██╗    ██╔════╝██╔════╝██╔════╝╚══██╔══╝██║██╔═══██╗████╗  ██║██╔════╝
    ██║   ██║   ██║    ███████╗█████╗  ██║        ██║   ██║██║   ██║██╔██╗ ██║███████╗
    ██║   ██║   ██║    ╚════██║██╔══╝  ██║        ██║   ██║██║   ██║██║╚██╗██║╚════██║
    ██║   ╚██████╔╝    ███████║███████╗╚██████╗   ██║   ██║╚██████╔╝██║ ╚████║███████║
    ╚═╝    ╚═════╝     ╚══════╝╚══════╝ ╚═════╝   ╚═╝   ╚═╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝
*/

fn to_sections(items: Vec<MenuItem>) -> Vec<Vec<MenuItem>> {
    let mut sections: Vec<Vec<MenuItem>> = Vec::new();
    let mut section: Vec<MenuItem> = Vec::new();
    for item in items {
        if item.label.is_none() {
            sections.push(section);
            section = Vec::new();
        }
        section.push(item);
    }
    sections.push(section);
    sections
}
