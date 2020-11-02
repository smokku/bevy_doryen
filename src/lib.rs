use bevy::{
    app::startup_stage,
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    type_registry::TypeUuid,
    utils::BoxedFuture,
    winit::WinitWindows,
};
use doryen_rs::{set_texture_params, Program, DORYEN_FS, DORYEN_VS};
use std::ops::{Deref, DerefMut};

pub use doryen_rs::{self as doryen, Color};

#[allow(dead_code)]
pub struct Console {
    con: doryen::Console,
    x: f32,
    y: f32,
}

impl Console {
    pub fn new(x: f32, y: f32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            con: doryen::Console::new(width, height),
        }
    }

    pub fn pixel_to_pos(&self, x: f32, y: f32, width: u32, height: u32) -> (f32, f32) {
        (
            x / width as f32 * self.con.get_width() as f32,
            y / height as f32 * self.con.get_height() as f32,
        )
    }
}

impl Deref for Console {
    type Target = doryen::Console;
    fn deref(&self) -> &doryen::Console {
        &self.con
    }
}

impl DerefMut for Console {
    fn deref_mut(&mut self) -> &mut doryen::Console {
        &mut self.con
    }
}

pub struct Doryen {
    consoles: Vec<Console>,
    gl: uni_gl::WebGLRenderingContext,
    font_texture: uni_gl::WebGLTexture,
    font_asset: Handle<Font>,
    font_is_loading: bool,
    pub font_width: u32,
    pub font_height: u32,
    pub char_width: u32,
    pub char_height: u32,
    program: Program,
}

impl Doryen {
    pub fn con(&self) -> &Console {
        &self.consoles[0]
    }

    pub fn con_mut(&mut self) -> &mut Console {
        &mut self.consoles[0]
    }

    pub fn load_font<S: AsRef<str>>(&mut self, asset_server: Ref<AssetServer>, font_file: S) {
        self.font_asset = asset_server.load(font_file.as_ref());
        self.font_is_loading = true;
        log::debug!("Loading Font {:?}", self.font_asset);
    }

    fn font_loaded(&mut self, font: &mut Font) {
        self.font_is_loading = false;

        let img = font.img.take().unwrap();
        if font.char_width != 0 {
            self.char_width = font.char_width;
            self.char_height = font.char_height;
        } else {
            self.char_width = img.width() as u32 / 16;
            self.char_height = img.height() as u32 / 16;
        }
        self.font_width = img.width() as u32;
        self.font_height = img.height() as u32;
        log::debug!(
            "font size: {:?} char size: {:?}",
            (self.font_width, self.font_height),
            (self.char_width, self.char_height)
        );
        self.gl.active_texture(0);
        self.gl.bind_texture(&self.font_texture);
        self.gl.tex_image2d(
            uni_gl::TextureBindPoint::Texture2d, // target
            0,                                   // level
            img.width() as u16,                  // width
            img.height() as u16,                 // height
            uni_gl::PixelFormat::Rgba,           // format
            uni_gl::PixelType::UnsignedByte,     // type
            &*img,                               // data
        );

        self.program.bind(
            &self.gl,
            &self.consoles[0].con,
            self.font_width,
            self.font_height,
            self.char_width,
            self.char_height,
        );
        self.program
            .set_texture(&self.gl, uni_gl::WebGLTexture(self.font_texture.0));
    }

    fn render(&mut self) {
        self.program
            .render_primitive(&self.gl, &self.consoles[0].con);
    }
}

#[derive(Default)]
pub struct DoryenPlugin;
pub static DRAW_STAGE: &str = "doryen_draw_stage";

impl Plugin for DoryenPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<Font>()
            .init_asset_loader::<FontLoader>()
            .add_startup_system_to_stage(startup_stage::PRE_STARTUP, init.thread_local_system())
            .add_stage_before(stage::LAST, DRAW_STAGE)
            .add_system_to_stage(DRAW_STAGE, draw.thread_local_system());
    }
}

#[derive(Default)]
pub struct DoryenConfig {
    pub console_width: u32,
    pub console_height: u32,
    pub font: String,
}

fn init(_world: &mut World, resources: &mut Resources) {
    let windows = resources.get::<Windows>().unwrap();
    let window = windows.get_primary().unwrap();
    let winit_windows = resources.get::<WinitWindows>().unwrap();
    let winit_window = winit_windows.get_window(window.id()).unwrap();
    log::debug!("winit_window {:?}", winit_window.inner_size(),);

    use glutin::platform::unix::RawContextExt;
    use glutin::ContextBuilder;
    use winit::platform::unix::WindowExtUnix;

    let windowed_context = unsafe {
        let xconn = winit_window.xlib_xconnection().unwrap();
        let xwindow = winit_window.xlib_window().unwrap();
        let mut context_builder = ContextBuilder::new().with_vsync(window.vsync());
        context_builder.pf_reqs.alpha_bits = None;
        context_builder.pf_reqs.srgb = true;
        let raw_context = context_builder
            .build_raw_x11_context(xconn, xwindow)
            .unwrap();
        raw_context.make_current().unwrap()
    };
    log::debug!(
        "Pixel format of the window's GL context: {:?}",
        windowed_context.get_pixel_format()
    );

    let gl = uni_gl::WebGLRenderingContext::new(Box::new(|s| windowed_context.get_proc_address(s)));
    gl.viewport(0, 0, window.width(), window.height());
    gl.enable(uni_gl::Flag::Blend as i32);
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(uni_gl::BufferBit::Color);
    gl.blend_equation(uni_gl::BlendEquation::FuncAdd);
    gl.blend_func(
        uni_gl::BlendMode::SrcAlpha,
        uni_gl::BlendMode::OneMinusSrcAlpha,
    );

    std::mem::drop(winit_window);
    std::mem::drop(winit_windows);
    std::mem::drop(window);
    std::mem::drop(windows);

    resources.insert_thread_local(windowed_context);

    let program = Program::new(&gl, DORYEN_VS, DORYEN_FS);

    let font_texture = gl.create_texture();
    gl.active_texture(0);
    gl.bind_texture(&font_texture);
    set_texture_params(&gl, true);

    let (mut console_width, mut console_height, mut font) =
        if let Some(config) = resources.get::<DoryenConfig>() {
            (config.console_width, config.console_height, config.font.clone())
        } else {
            (0, 0, String::new())
        };
    if console_width == 0 {
        console_width = 80;
    }
    if console_height == 0 {
        console_height = 25;
    }

    let mut doryen = Doryen {
        consoles: vec![Console::new(0.0, 0.0, console_width, console_height)],
        gl,
        font_texture,
        font_asset: Handle::default(),
        font_is_loading: false,
        font_width: 0,
        font_height: 0,
        char_width: 0,
        char_height: 0,
        program,
    };

    if font.len() == 0 {
        font = String::from("terminal_8x8.png");
    };

    doryen.load_font(resources.get::<AssetServer>().unwrap(), font);

    resources.insert(doryen);
}

fn draw(_world: &mut World, resources: &mut Resources) {
    let mut doryen = resources.get_mut::<Doryen>().unwrap();
    if doryen.font_is_loading {
        let mut fonts = resources.get_mut::<Assets<Font>>().unwrap();
        if let Some(font) = fonts.get_mut(&doryen.font_asset) {
            doryen.font_loaded(font);
        }
    }

    if doryen.font_width > 0
        && doryen.font_height > 0
        && doryen.char_width > 0
        && doryen.char_height > 0
    {
        let windows = resources.get::<Windows>().unwrap();
        let window = windows.get_primary().unwrap();
        doryen.gl.viewport(0, 0, window.width(), window.height());

        doryen.render();

        use glutin::{ContextWrapper, PossiblyCurrent};
        let windowed_context = resources
            .get_thread_local::<ContextWrapper<PossiblyCurrent, ()>>()
            .unwrap();
        windowed_context.swap_buffers().unwrap();
    }
}

#[derive(TypeUuid)]
#[uuid = "30bc3c3a-72ec-4a7e-91c8-a7133069da78"]
pub struct Font(doryen::FontLoader);

impl Deref for Font {
    type Target = doryen::FontLoader;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Font {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Font {
    pub fn new_from_bytes(buf: &[u8]) -> Font {
        let mut font_loader = doryen::FontLoader::new();
        font_loader.load_font_bytes(buf);
        Font(font_loader)
    }
}

#[derive(Default)]
pub struct FontLoader;

impl AssetLoader for FontLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut font = Font::new_from_bytes(bytes);
            font.set_size_from_name(load_context.path().to_str().unwrap_or_default());
            load_context.set_default_asset(LoadedAsset::new(font));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["png"]
    }
}
