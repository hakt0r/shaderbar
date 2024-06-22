use crate::gl::render::renderer;
use glium::{
    texture::{texture2d::Texture2d, RawImage2d},
    uniforms::AsUniformValue,
};
use glob::glob;
use std::{io::Cursor, u8};

// store images in a 64x64 texture
// store the name of the image to map to an id when updating the texture

pub struct IconState {
    pub raw: RawImage2d<'static, u8>,
    pub texture: Texture2d,
    pub width: u32,
    pub height: u32,
    pub name: Vec<String>,
}

pub fn icon_state() -> &'static mut IconState {
    static mut ICON_STATE: Option<IconState> = None;
    let context = &renderer().unwrap().context;
    unsafe {
        return ICON_STATE.get_or_insert_with(|| IconState {
            raw: RawImage2d {
                data: std::borrow::Cow::Owned(vec![0; 64 * 64 * 4]),
                width: 64,
                height: 64,
                format: glium::texture::ClientFormat::U8U8U8U8,
            },
            texture: Texture2d::empty(&*context, 64, 64).unwrap(),
            width: 0,
            height: 0,
            name: Vec::new(),
        });
    }
}

/*
  █████╗ ██████╗ ██████╗
 ██╔══██╗██╔══██╗██╔══██╗
 ███████║██║  ██║██║  ██║
 ██╔══██║██║  ██║██║  ██║
 ██║  ██║██████╔╝██████╔╝
 ╚═╝  ╚═╝╚═════╝ ╚═════╝
*/

pub fn add(id: String, name: String) {
    let state = icon_state();
    state.name.push(id);
    eprintln!("Loading icon: {}", name);
    let path = resolve(name.clone());
    if path.is_empty() {
        eprintln!("Icon not found: {}", name);
        return;
    }
    let id = (state.name.len() - 1) as u32;
    // load texture, icon, add icon to texture
    let image = image::load(
        Cursor::new(std::fs::read(path).unwrap()),
        image::ImageFormat::Png,
    )
    .unwrap()
    .to_rgba8();
    let (width, height) = image.dimensions();
    let raw = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), (width, height));
    state.texture.write(icon_rect(id), raw);
    unsafe {
        state.texture.as_uniform_value();
    }
}

/*
 ██████╗ ███████╗███╗   ███╗ ██████╗ ██╗   ██╗███████╗
 ██╔══██╗██╔════╝████╗ ████║██╔═══██╗██║   ██║██╔════╝
 ██████╔╝█████╗  ██╔████╔██║██║   ██║██║   ██║█████╗
 ██╔══██╗██╔══╝  ██║╚██╔╝██║██║   ██║╚██╗ ██╔╝██╔══╝
 ██║  ██║███████╗██║ ╚═╝ ██║╚██████╔╝ ╚████╔╝ ███████╗
 ╚═╝  ╚═╝╚══════╝╚═╝     ╚═╝ ╚═════╝   ╚═══╝  ╚══════╝
*/

pub fn remove(name: String) {
    let state = icon_state();
    let index_search = state.name.iter().position(|x| *x == name);
    if index_search.is_none() {
        eprintln!("Icon not found: {}", name);
        return;
    }
    let index = index_search.unwrap();
    state.name.remove(index);
    let id = index as u32;
    eprintln!("Removing icon[{}] id={}", name, id);
    state.texture.write(
        icon_rect(id),
        RawImage2d {
            data: std::borrow::Cow::Owned(vec![0; 16 * 16 * 4]),
            width: 16,
            height: 16,
            format: glium::texture::ClientFormat::U8U8U8U8,
        },
    );
}

/*
 ██████╗ ███████╗███████╗ ██████╗ ██╗    ██╗   ██╗███████╗    ██╗ ██████╗ ██████╗ ███╗   ██╗
 ██╔══██╗██╔════╝██╔════╝██╔═══██╗██║    ██║   ██║██╔════╝    ██║██╔════╝██╔═══██╗████╗  ██║
 ██████╔╝█████╗  ███████╗██║   ██║██║    ██║   ██║█████╗      ██║██║     ██║   ██║██╔██╗ ██║
 ██╔══██╗██╔══╝  ╚════██║██║   ██║██║    ╚██╗ ██╔╝██╔══╝      ██║██║     ██║   ██║██║╚██╗██║
 ██║  ██║███████╗███████║╚██████╔╝███████╗╚████╔╝ ███████╗    ██║╚██████╗╚██████╔╝██║ ╚████║
 ╚═╝  ╚═╝╚══════╝╚══════╝ ╚═════╝ ╚══════╝ ╚═══╝  ╚══════╝    ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝
*/

pub fn resolve(name: String) -> String {
    let mut path = String::from("/usr/share/icons/*/16x16/**/");
    path.push_str(&name);
    path.push_str(".png");
    let mut resolved = String::from("");
    for entry in glob(&path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                resolved = path.display().to_string();
                break;
            }
            Err(e) => println!("{:?}", e),
        }
    }
    return resolved;
}

/*
 ████████╗ ██████╗  ██████╗ ██╗     ███████╗
 ╚══██╔══╝██╔═══██╗██╔═══██╗██║     ██╔════╝
    ██║   ██║   ██║██║   ██║██║     ███████╗
    ██║   ██║   ██║██║   ██║██║     ╚════██║
    ██║   ╚██████╔╝╚██████╔╝███████╗███████║
    ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝╚══════╝
*/

#[inline]
fn icon_rect(id: u32) -> glium::Rect {
    glium::Rect {
        left: id % 4 * 16,
        bottom: id / 4 * 16,
        width: 16,
        height: 16,
    }
}
