use super::uniform::initialize_uniforms;
use crate::gl::tools::read_shader;
use crate::gl::uniform::{default_index_buffer, default_vertex_buffer, SensorValues, Vertex};
use color_eyre::owo_colors::OwoColorize;
use glium::{program, uniform, uniforms::UniformBuffer, Frame, IndexBuffer, Surface, VertexBuffer};
use gtk4::cairo::{Context, Format, ImageSurface};
use gtk4::subclass::gl_area::GLAreaImpl;
use gtk4::{glib, prelude::*, subclass::prelude::*};
use std::io::BufReader;
use std::{cell::RefCell, rc::Rc};
pub struct Renderer {
    pub context: Rc<glium::backend::Context>,
    pub triangles: VertexBuffer<Vertex>,
    pub index: IndexBuffer<u16>,
    pub buffer: UniformBuffer<SensorValues>,
    pub program: glium::Program,
    pub frame: u64,
    pub font_texture: Rc<RefCell<Option<glium::texture::Texture2d>>>,
}

impl Renderer {
    fn new(context: Rc<glium::backend::Context>) -> Self {
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
                std::process::exit(1);
            });

        let buffer = initialize_uniforms(context.clone());
        Renderer {
            buffer,
            context,
            frame: 0,
            index,
            program,
            triangles,
            font_texture: Rc::new(RefCell::new(None)),
        }
    }

    fn _prepare_textures(&self) {
        let file_exists = std::path::Path::new("texture.png").exists();
        if !file_exists {
            let mut file = std::fs::File::create("texture.png").unwrap();
            let surface = ImageSurface::create(Format::ARgb32, 256, 256).unwrap();
            let cr = Context::new(&surface).unwrap();
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            _ = cr.paint();
            _ = cr.font_options().unwrap().antialias().green();
            let font = gtk4::cairo::FontFace::toy_create(
                "SauceCodePro Nerd Font",
                gtk4::cairo::FontSlant::Normal,
                gtk4::cairo::FontWeight::Normal,
            )
            .unwrap();
            cr.set_font_face(&font);
            cr.set_font_size(8.0);
            cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
            cr.move_to(0.0, 8.0);
            _ = cr.show_text(
                "0123456789:;<=?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz",
            );
            _ = surface.write_to_png(&mut file);
        }
        let image = image::load(
            BufReader::new(std::fs::File::open("texture.png").unwrap()),
            image::ImageFormat::Png,
        )
        .unwrap()
        .to_rgba8();
        let image_dimensions = image.dimensions();
        let image =
            glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::texture::Texture2d::new(&self.context, image).unwrap();
        let mut font_texture = self.font_texture.borrow_mut();
        *font_texture = Some(texture);
    }

    fn draw(&mut self) {
        let dimensions = self.context.get_framebuffer_dimensions();

        let mut frame = Frame::new(self.context.clone(), dimensions);

        {
            let mut map = self.buffer.map();
            map.width = dimensions.0;
        }

        frame
            .draw(
                &self.triangles,
                &self.index,
                &self.program,
                &uniform! { sensors: &*self.buffer },
                &Default::default(),
            )
            .unwrap();
        frame.finish().unwrap();

        self.frame += 1;
    }
}

#[derive(Default)]
pub struct GliumGLArea {
    pub renderer: RefCell<Option<Renderer>>,
}

#[glib::object_subclass]
impl ObjectSubclass for GliumGLArea {
    const NAME: &'static str = "GliumGLArea";
    type Type = crate::gl::GliumGLArea;
    type ParentType = gtk4::GLArea;
}

impl ObjectImpl for GliumGLArea {}

impl WidgetImpl for GliumGLArea {
    fn realize(&self) {
        self.parent_realize();

        let widget = self.obj();

        if widget.as_ref().error().is_some() {
            return;
        }

        let context =
            unsafe { glium::backend::Context::new(widget.clone(), true, Default::default()) }
                .unwrap();
        unsafe {
            RENDERER = Some(Renderer::new(context));
            RENDERER.as_mut().unwrap()._prepare_textures();
        }
    }

    fn unrealize(&self) {
        self.parent_unrealize();
    }
}

impl GLAreaImpl for GliumGLArea {
    fn render(&self, _context: &gtk4::gdk::GLContext) -> glib::Propagation {
        let renderer = renderer();
        if renderer.is_some() {
            renderer.unwrap().draw();
        } else {
            eprintln!("Renderer not initialized");
        }
        glib::Propagation::Stop
    }
}

static mut RENDERER: Option<Renderer> = None;

pub fn renderer() -> Option<&'static mut Renderer> {
    unsafe { return RENDERER.as_mut() }
}
