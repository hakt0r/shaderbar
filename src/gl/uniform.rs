use crate::sensors;
use glium::backend::Context;
use glium::{
    buffer::Mapping, implement_uniform_block, implement_vertex, index::PrimitiveType, uniforms::*,
    IndexBuffer, VertexBuffer,
};
use std::rc::Rc;

pub const HISTORY_SIZE: usize = 256;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SensorValues {
    pub width: u32,
    pub time: u32,
    pub gauge_count: u32,
    pub gauge_value: [u32; 6],
    pub gauge_color: [u32; 6],
    pub load_ptr: u32,
    pub load_count: u32,
    pub load_color: [u32; 24],
    pub load: [u32; 2048],
    pub text: [u32; 256],
}

implement_uniform_block!(
    SensorValues,
    width,
    time,
    gauge_count,
    gauge_value,
    gauge_color,
    load_ptr,
    load_count,
    load_color,
    load,
    text
);

#[inline]
pub fn u32e4(a: u8, b: u8, c: u8, d: u8) -> u32 {
    return (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | d as u32;
}

#[inline]
pub fn u32e3(a: u8, b: u8, c: u8) -> u32 {
    return (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | 0 as u32;
}

#[inline]
pub fn u32d4(page: u32) -> Vec<u8> {
    return vec![
        (page >> 24) as u8,
        (page >> 16) as u8,
        (page >> 8) as u8,
        page as u8,
    ];
}

const RED: u32 = (255 as u32) << 24 | (0 as u32) << 16 | (0 as u32) << 8 | 0 as u32;
const ORANGE: u32 = (255 as u32) << 24 | (100 as u32) << 16 | (0 as u32) << 8 | 0 as u32;
const YELLOW: u32 = (255 as u32) << 24 | (255 as u32) << 16 | (0 as u32) << 8 | 0 as u32;
const GREEN: u32 = (0 as u32) << 24 | (255 as u32) << 16 | (0 as u32) << 8 | 0 as u32;
const BLUE: u32 = (0 as u32) << 24 | (0 as u32) << 16 | (255 as u32) << 8 | 0 as u32;

pub fn initialize_uniforms(context: Rc<Context>) -> UniformBuffer<SensorValues> {
    let mut buffer: UniformBuffer<SensorValues> = UniformBuffer::empty(&context.clone()).unwrap();
    {
        let s = sensors();
        let mut map = buffer.map();
        let frame = 0;
        let ptr: usize = frame as usize % HISTORY_SIZE;
        let cpus = s.cpu_count as usize;
        map.width = 1920;
        map.load = [0u32; 2048];
        map.load_count = cpus as u32 + 2;
        map.load_ptr = ptr as u32;
        map.load_color = [u32e3(1, 1, 1); 24];
        map.gauge_count = 5;
        map.gauge_value = [0u32; 6];
        map.gauge_color = [RED, RED, BLUE, YELLOW, ORANGE, YELLOW];
        // 60 seconds with milliseconds
        map.time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u32;
        // format current time as string
        map.text = [64u32; 256];
        encode_text(
            &mut map.text,
            &format!("{:02}:{:02}:{:02}", s.hour, s.minute, s.second),
        );
    }

    return buffer;
}

pub fn encode_text(text: &mut [u32; 256], string: &str) {
    #[inline]
    fn byte_or_0(bytes: &[u8], index: usize) -> u8 {
        if index >= bytes.len() {
            return 0;
        }
        return bytes[index];
    }
    let bytes = string.as_bytes();
    for i in 0..64 {
        let o = i * 4;
        let b1 = byte_or_0(bytes, o);
        let b2 = byte_or_0(bytes, o + 1);
        let b3 = byte_or_0(bytes, o + 2);
        let b4 = byte_or_0(bytes, o + 3);
        text[i] = u32e4(b1, b2, b3, b4);
    }
}

pub fn update_uniforms() {
    let renderer = super::render::renderer();
    if renderer.is_none() {
        return;
    }
    let renderer = renderer.unwrap();
    let mut map: Mapping<SensorValues> = renderer.buffer.map();

    let s = sensors();
    let frame = renderer.frame;
    let ptr: usize = frame as usize % HISTORY_SIZE;

    map.load_ptr = ptr as u32;

    map.time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u32;

    encode_text(
        &mut map.text,
        &format!("{:_>216}{:02}:{:02}:{:02}", "", s.hour, s.minute, s.second),
    );

    #[inline]
    fn write_pixel(map: &mut Mapping<SensorValues>, y: usize, x: usize, value: u8) {
        let page_index = y * 64 + (x as f64 / 4.0).floor() as usize;
        let byte_index = x % 4;
        let mut page = u32d4(map.load[page_index]);
        page[byte_index] = value;
        map.load[page_index] = u32e4(page[0], page[1], page[2], page[3]);
    }

    for (i, usage) in s.cpu_load.iter().enumerate() {
        write_pixel(&mut map, i, ptr, *usage);
    }

    let cpus: usize = s.cpu_count as usize;
    write_pixel(&mut map, cpus + 0, ptr, s.cpu_temp as u8);
    write_pixel(&mut map, cpus + 1, ptr, s.gpu_temp as u8);
    write_pixel(&mut map, cpus + 2, ptr, s.cpu_fan as u8);
    write_pixel(&mut map, cpus + 3, ptr, s.gpu_fan as u8);

    map.gauge_value[0] = s.bat as u32;
    map.gauge_value[1] = 255;
    map.gauge_color[0] = match s.bat {
        0..=10 => RED,
        11..=20 => ORANGE,
        21..=30 => YELLOW,
        _ => GREEN,
    };
    map.gauge_color[1] = match s.bat_status {
        1 => GREEN,
        _ => RED,
    };
    map.gauge_value[2] = s.cpu_temp as u32;
    map.gauge_value[3] = s.cpu_fan as u32;
    map.gauge_value[4] = s.gpu_temp as u32;
    map.gauge_value[5] = s.gpu_fan as u32;
}

/*
 ██╗███╗   ██╗██████╗ ███████╗██╗  ██╗    ██████╗ ██╗   ██╗███████╗███████╗███████╗██████╗
 ██║████╗  ██║██╔══██╗██╔════╝╚██╗██╔╝    ██╔══██╗██║   ██║██╔════╝██╔════╝██╔════╝██╔══██╗
 ██║██╔██╗ ██║██║  ██║█████╗   ╚███╔╝     ██████╔╝██║   ██║█████╗  █████╗  █████╗  ██████╔╝
 ██║██║╚██╗██║██║  ██║██╔══╝   ██╔██╗     ██╔══██╗██║   ██║██╔══╝  ██╔══╝  ██╔══╝  ██╔══██╗
 ██║██║ ╚████║██████╔╝███████╗██╔╝ ██╗    ██████╔╝╚██████╔╝██║     ██║     ███████╗██║  ██║
 ╚═╝╚═╝  ╚═══╝╚═════╝ ╚══════╝╚═╝  ╚═╝    ╚═════╝  ╚═════╝ ╚═╝     ╚═╝     ╚══════╝╚═╝  ╚═╝

    All this work just to draw a rectangle on the screen. :D I love being an engineer.
    These are the edges of the rectangle. The rectangle is drawn using two triangles.

*/

pub fn default_index_buffer(context: &Rc<Context>) -> IndexBuffer<u16> {
    IndexBuffer::new(context, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3]).unwrap()
}

/*
 ██╗   ██╗███████╗██████╗ ████████╗███████╗██╗  ██╗    ██████╗ ██╗   ██╗███████╗███████╗███████╗██████╗
 ██║   ██║██╔════╝██╔══██╗╚══██╔══╝██╔════╝╚██╗██╔╝    ██╔══██╗██║   ██║██╔════╝██╔════╝██╔════╝██╔══██╗
 ██║   ██║█████╗  ██████╔╝   ██║   █████╗   ╚███╔╝     ██████╔╝██║   ██║█████╗  █████╗  █████╗  ██████╔╝
 ╚██╗ ██╔╝██╔══╝  ██╔══██╗   ██║   ██╔══╝   ██╔██╗     ██╔══██╗██║   ██║██╔══╝  ██╔══╝  ██╔══╝  ██╔══██╗
  ╚████╔╝ ███████╗██║  ██║   ██║   ███████╗██╔╝ ██╗    ██████╔╝╚██████╔╝██║     ██║     ███████╗██║  ██║
   ╚═══╝  ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚══════╝╚═╝  ╚═╝    ╚═════╝  ╚═════╝ ╚═╝     ╚═╝     ╚══════╝╚═╝  ╚═╝

   These are the vertices of the rectangle. The rectangle is drawn using two triangles.

*/

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub fn default_vertex_buffer(context: &Rc<Context>) -> VertexBuffer<Vertex> {
    VertexBuffer::new(
        context,
        &[
            Vertex {
                position: [-1.0, -1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
                tex_coords: [1.0, 0.0],
            },
        ],
    )
    .unwrap()
}
