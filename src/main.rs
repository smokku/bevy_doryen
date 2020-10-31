use bevy::{
    app::startup_stage,
    asset::{AssetLoader, LoadContext, LoadedAsset},
    input::keyboard::{ElementState, KeyboardInput},
    prelude::*,
    type_registry::TypeUuid,
    utils::BoxedFuture,
    winit::WinitWindows,
};
use doryen::{set_texture_params, Program, TextAlign, DORYEN_FS, DORYEN_VS};
use std::ops::{Deref, DerefMut};

const CONSOLE_WIDTH: u32 = 80;
const CONSOLE_HEIGHT: u32 = 60;

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
    pub fn load_font(&mut self, asset_server: Ref<AssetServer>, font_file: &str) {
        self.font_asset = asset_server.load(font_file);
        self.font_is_loading = true;
        println!("Loading Font {:?}", self.font_asset);
    }

    fn font_loaded(&mut self, font: &mut Font) {
        println!("Font: {}x{}", font.char_width, font.char_height);
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
        println!(
            "font size: {:?} char size: {:?}\n",
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
}

struct MyRoguelike {
    player_pos: (i32, i32),
    mouse_pos: (f32, f32),
}

impl MyRoguelike {
    pub fn new() -> Self {
        Self {
            player_pos: ((CONSOLE_WIDTH / 2) as i32, (CONSOLE_HEIGHT / 2) as i32),
            mouse_pos: (0.0, 0.0),
        }
    }
}

pub fn main() {
    log::info!("Starting basic example");

    App::build()
        .add_default_plugins()
        .add_asset::<Font>()
        .init_asset_loader::<FontLoader>()
        .add_startup_system_to_stage(startup_stage::PRE_STARTUP, init.thread_local_system())
        .add_resource(MyRoguelike::new())
        .add_system_to_stage(stage::PRE_UPDATE, input.system()) // game inputs
        .add_system_to_stage(stage::POST_UPDATE, render.system()) // render game state to console
        .add_system_to_stage(stage::POST_UPDATE, draw.system()) // engine - consoles to screen
        .run();
}

fn init(_world: &mut World, resources: &mut Resources) {
    let windows = resources.get::<Windows>().unwrap();
    let window = windows.get_primary().unwrap();
    let winit_windows = resources.get::<WinitWindows>().unwrap();
    let winit_window = winit_windows.get_window(window.id()).unwrap();

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
    println!(
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

    let program = Program::new(&gl, DORYEN_VS, DORYEN_FS);

    let font_texture = gl.create_texture();
    gl.active_texture(0);
    gl.bind_texture(&font_texture);
    set_texture_params(&gl, true);

    let mut con = Console::new(0.0, 0.0, CONSOLE_WIDTH, CONSOLE_HEIGHT);
    con.register_color("white", (255, 255, 255, 255));
    con.register_color("red", (255, 92, 92, 255));
    con.register_color("blue", (192, 192, 255, 255));

    let mut doryen = Doryen {
        consoles: vec![con],
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

    doryen.load_font(resources.get::<AssetServer>().unwrap(), "terminal_8x8.png");

    resources.insert(doryen);
}

fn render(game: Res<MyRoguelike>, mut doryen: ResMut<Doryen>) {
    let con = &mut doryen.consoles[0];
    con.rectangle(
        0,
        0,
        CONSOLE_WIDTH,
        CONSOLE_HEIGHT,
        Some((128, 128, 128, 255)),
        Some((0, 0, 0, 255)),
        Some('.' as u16),
    );
    con.area(
        10,
        10,
        5,
        5,
        Some((255, 64, 64, 255)),
        Some((128, 32, 32, 255)),
        Some('&' as u16),
    );
    con.ascii(game.player_pos.0, game.player_pos.1, '@' as u16);
    con.fore(game.player_pos.0, game.player_pos.1, (255, 255, 255, 255));
    con.print_color(
        (CONSOLE_WIDTH / 2) as i32,
        (CONSOLE_HEIGHT - 1) as i32,
        "#[red]arrows#[white] : move",
        TextAlign::Center,
        None,
    );
    con.print_color(
        (CONSOLE_WIDTH / 2) as i32,
        (CONSOLE_HEIGHT - 3) as i32,
        &format!(
            "#[white]Mouse coordinates: #[red]{}, {}",
            game.mouse_pos.0, game.mouse_pos.1
        ),
        TextAlign::Center,
        None,
    );
    con.print_color(
        5,
        5,
        "#[blue]This blue text contains a #[red]red#[] word",
        TextAlign::Left,
        None,
    );
    con.back(
        game.mouse_pos.0 as i32,
        game.mouse_pos.1 as i32,
        (255, 255, 255, 255),
    );
    // println!(
    //     "{}x{} / {}x{}",
    //     game.player_pos.0, game.player_pos.1, game.mouse_pos.0 as i32, game.mouse_pos.1 as i32
    // );
}

fn draw(mut doryen: ResMut<Doryen>, mut fonts: ResMut<Assets<Font>>) {
    if doryen.font_is_loading {
        if let Some(font) = fonts.get_mut(&doryen.font_asset) {
            doryen.font_loaded(font);
        }
    }
    // if let Some(window) = windows.get_primary() {
    //     // let winit_window = winit_windows.get_window(window.id()).unwrap();
    //     // println!("{:?}", *winit_window);
    //     std::process::exit(0);

    //     // app.add_system_to_stage(
    //     //     bevy_render::stage::RENDER,
    //     //     render_system.thread_local_system(),
    //     // )
    //     // https://github.com/jice-nospam/doryen-rs/blob/master/src/app.rs#L438
    //     //
    //     // https://github.com/bevyengine/bevy/blob/0dba0fe45f60cf06e802e5ff08710290ad7870d6/crates/bevy_wgpu/src/lib.rs#L24
    //     // update: https://github.com/bevyengine/bevy/blob/0dba0fe45f60cf06e802e5ff08710290ad7870d6/crates/bevy_wgpu/src/wgpu_renderer.rs#L112
    //     // https://github.com/bevyengine/bevy/blob/master/crates/bevy_wgpu/src/wgpu_renderer.rs#L66
    //     // winit_window + https://github.com/rust-windowing/glutin/blob/master/glutin_examples/examples/raw_context.rs#L55
    //     // https://github.com/grovesNL/glow/blob/main/examples/hello/src/main.rs#L73
    //     // + pub window_surfaces: Arc<RwLock<HashMap<WindowId, wgpu::Surface>>>,
    // }
}

#[derive(Default)]
struct EventsState {
    keyboard_input_event_reader: EventReader<KeyboardInput>,
    cursor_moved_event_reader: EventReader<CursorMoved>,
}

fn input(
    mut state: Local<EventsState>,
    keyboard_input_events: Res<Events<KeyboardInput>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    mut game: ResMut<MyRoguelike>,
) {
    for event in state
        .keyboard_input_event_reader
        .iter(&keyboard_input_events)
    {
        if event.state == ElementState::Pressed {
            match event.key_code {
                Some(KeyCode::Left) => {
                    game.player_pos.0 = (game.player_pos.0 - 1).max(1);
                }
                Some(KeyCode::Right) => {
                    game.player_pos.0 = (game.player_pos.0 + 1).min(CONSOLE_WIDTH as i32 - 2);
                }
                Some(KeyCode::Up) => {
                    game.player_pos.1 = (game.player_pos.1 - 1).max(1);
                }
                Some(KeyCode::Down) => {
                    game.player_pos.1 = (game.player_pos.1 + 1).min(CONSOLE_HEIGHT as i32 - 2);
                }
                _ => {}
            }
        }
    }
    for event in state.cursor_moved_event_reader.iter(&cursor_moved_events) {
        game.mouse_pos = (event.position.x(), event.position.y());
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
