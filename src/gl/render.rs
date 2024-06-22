use super::uniform::initialize_uniforms;
use crate::gl::tools::read_shader;
use crate::gl::uniform::{default_index_buffer, default_vertex_buffer, SensorValues, Vertex};
use crate::tray::icon::icon_state;
use glib::Propagation;
use glium::backend::Context as GliumContext;
use glium::{
    program, texture::*, uniform, uniforms::UniformBuffer, Frame, IndexBuffer, Surface,
    VertexBuffer,
};
use gtk4::{
    gdk::GLContext, prelude::*, subclass::gl_area::GLAreaImpl, subclass::prelude::*, GLArea,
};
use std::io::Cursor;
use std::{cell::RefCell, process::exit, rc::Rc};

pub struct Renderer {
    pub context: Rc<GliumContext>,
    pub triangles: VertexBuffer<Vertex>,
    pub index: IndexBuffer<u16>,
    pub buffer: UniformBuffer<SensorValues>,
    pub program: glium::Program,
    pub frame: u64,
    pub font: Texture2d,
}

impl Renderer {
    fn new(context: Rc<GliumContext>) -> Self {
        let vertex = read_shader("src/gl/vertex.glsl");
        let fragment = read_shader("src/gl/fragment.glsl");
        let index = default_index_buffer(&context);
        let triangles = default_vertex_buffer(&context);

        let program = program!(&context, 140 => { vertex: &vertex, fragment: &fragment })
            .unwrap_or_else(|err| {
                println!(
                    "\x1b[31m\nFailed to create program:\n\x1b[0m \x1b[33m{}\x1b[0m",
                    err
                );
                exit(1);
            });

        let buffer = initialize_uniforms(context.clone());

        let image = image::load(
            Cursor::new(&include_bytes!("../font.png")[..]),
            image::ImageFormat::Png,
        )
        .unwrap()
        .to_rgba8();

        let dimensions = image.dimensions();
        eprintln!("Dimensions: {:?}", dimensions);
        let image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions);
        let texture = Texture2d::new(&context, image).unwrap();

        Renderer {
            buffer,
            context,
            frame: 0,
            index,
            program,
            triangles,
            font: texture,
        }
    }

    fn draw(&mut self) {
        let texture = &icon_state().texture;
        let dimensions = self.context.get_framebuffer_dimensions();
        let mut frame = Frame::new(self.context.clone(), dimensions);
        {
            let mut map = self.buffer.map();
            map.width = dimensions.0;
        }
        unsafe { self.context.as_ref().rebind_textures() }
        {
            frame
                .draw(
                    &self.triangles,
                    &self.index,
                    &self.program,
                    &uniform! {
                        sensors: &*self.buffer,
                        icon_fix: texture,
                        icon: texture,
                        font: &self.font,
                    },
                    &Default::default(),
                )
                .unwrap();
        }
        frame.finish().unwrap();
        self.frame += 1;
        unsafe { self.context.as_ref().rebind_textures() }
    }
}

#[derive(Default)]
pub struct GliumGLArea {
    pub renderer: RefCell<Option<Renderer>>,
    pub context: Option<Rc<GliumContext>>,
}

#[glib::object_subclass]
impl ObjectSubclass for GliumGLArea {
    const NAME: &'static str = "GliumGLArea";
    type Type = crate::gl::GliumGLArea;
    type ParentType = GLArea;
}

impl ObjectImpl for GliumGLArea {}

impl WidgetImpl for GliumGLArea {
    fn realize(&self) {
        self.parent_realize();

        let widget = self.obj();

        if widget.as_ref().error().is_some() {
            return;
        }
        unsafe {
            let context = GliumContext::new(widget.clone(), true, Default::default()).unwrap();
            RENDERER = Some(Renderer::new(context));
        }
    }

    fn unrealize(&self) {
        self.parent_unrealize();
    }
}

impl GLAreaImpl for GliumGLArea {
    fn render(&self, _context: &GLContext) -> Propagation {
        let renderer = renderer();
        if renderer.is_some() {
            renderer.unwrap().draw();
        } else {
            eprintln!("Renderer not initialized");
        }
        Propagation::Stop
    }
}

static mut RENDERER: Option<Renderer> = None;

pub fn renderer() -> Option<&'static mut Renderer> {
    unsafe { return RENDERER.as_mut() }
}
