use bevy::app::{App, AppBuilder, AppExit, EventReader, Events, Plugin};
use bevy::input::{
    keyboard::{ElementState, KeyboardInput},
    mouse::MouseButtonInput,
};
use bevy::math::Vec2;
use bevy::window::{
    CursorMoved, WindowCreated, WindowDescriptor, WindowId, WindowMode, WindowResized,
};
use doryen_rs::{App as DoryenApp, Engine, UpdateEvent};
use std::sync::Arc;

pub use doryen_rs::{self as doryen, AppOptions, DoryenApi};

mod converters;
use converters::*;

pub type InitFn = Arc<Box<dyn Fn(&mut dyn DoryenApi, &mut App) -> () + Send + Sync>>;
pub type RenderFn = Arc<Box<dyn Fn(&mut App, &mut dyn DoryenApi) -> () + Send + Sync>>;

#[derive(Default)]
pub struct DoryenPlugin;

impl Plugin for DoryenPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.set_runner(doryen_runner);
    }
}

#[derive(Default, Debug)]
pub struct Window {
    pub width: usize,
    pub height: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
}

impl Window {
    fn new(width: usize, height: usize) -> Self {
        Window {
            width,
            height,
            ..Window::default()
        }
    }
}

pub fn doryen_runner(app: App) {
    log::debug!("Entering Doryen event loop");

    let options = {
        let options = app.resources.get::<AppOptions>();
        if options.is_some() {
            let options = options.unwrap();
            AppOptions {
                console_width: options.console_width,
                console_height: options.console_height,
                screen_width: options.screen_width,
                screen_height: options.screen_height,
                window_title: options.window_title.clone(),
                font_path: options.font_path.clone(),
                vsync: options.vsync,
                fullscreen: options.fullscreen,
                show_cursor: options.show_cursor,
                resizable: options.resizable,
                intercept_close_request: options.intercept_close_request,
                max_fps: options.max_fps,
            }
        } else {
            let mut options = AppOptions::default();
            if let Some(desc) = app.resources.get::<WindowDescriptor>() {
                options.window_title = desc.title.clone();
                options.screen_width = desc.width;
                options.screen_height = desc.height;
                options.vsync = desc.vsync;
                options.fullscreen = match desc.mode {
                    WindowMode::Windowed => false,
                    WindowMode::BorderlessFullscreen | WindowMode::Fullscreen { .. } => true,
                };
                options.show_cursor = desc.cursor_visible;
                options.resizable = desc.resizable;
            }

            options
        }
    };

    let mut doryen_app = DoryenApp::new(options);
    doryen_app.set_engine(Box::new(BevyEngine::new(app)));
    doryen_app.run();
}

struct BevyEngine {
    app: App,
    app_exit_event_reader: EventReader<AppExit>,
    last_mouse_pos: (f32, f32),
}

impl BevyEngine {
    pub fn new(app: App) -> Self {
        let app_exit_event_reader = EventReader::<AppExit>::default();

        BevyEngine {
            app,
            app_exit_event_reader,
            last_mouse_pos: (0.0, 0.0),
        }
    }
}

impl Engine for BevyEngine {
    fn init(&mut self, api: &mut dyn DoryenApi) {
        let (width, height) = api.get_screen_size();

        self.app
            .resources
            .insert(Window::new(width as usize, height as usize));

        {
            let mut window_created_events = self
                .app
                .resources
                .get_mut::<Events<WindowCreated>>()
                .unwrap();
            window_created_events.send(WindowCreated {
                id: WindowId::primary(),
            });
        }

        self.app.initialize();

        let init_function = if let Some(fn_ref) = self.app.resources.get::<InitFn>() {
            Some((*fn_ref).clone())
        } else {
            None
        };

        if let Some(init_function) = init_function {
            init_function(api, &mut self.app);
        }
    }

    fn resize(&mut self, api: &mut dyn DoryenApi) {
        let (width, height) = api.get_screen_size();

        let mut window_resized_events = self
            .app
            .resources
            .get_mut::<Events<WindowResized>>()
            .unwrap();
        window_resized_events.send(WindowResized {
            id: WindowId::primary(),
            width: width as usize,
            height: height as usize,
        });
    }

    fn update(&mut self, api: &mut dyn DoryenApi) -> Option<UpdateEvent> {
        if let Some(app_exit_events) = self.app.resources.get_mut::<Events<AppExit>>() {
            if self
                .app_exit_event_reader
                .latest(&app_exit_events)
                .is_some()
            {
                return Some(UpdateEvent::Exit);
            }
        }

        let input = api.input();

        for key in input.keys_pressed() {
            let mut keyboard_input_events = self
                .app
                .resources
                .get_mut::<Events<KeyboardInput>>()
                .unwrap();
            let input_event = KeyboardInput {
                scan_code: 0,
                state: ElementState::Pressed,
                key_code: convert_key_str(key),
            };
            keyboard_input_events.send(input_event);
        }

        for key in input.keys_released() {
            let mut keyboard_input_events = self
                .app
                .resources
                .get_mut::<Events<KeyboardInput>>()
                .unwrap();
            let input_event = KeyboardInput {
                scan_code: 0,
                state: ElementState::Released,
                key_code: convert_key_str(key),
            };
            keyboard_input_events.send(input_event);
        }

        for button in 0..3 {
            if input.mouse_button_pressed(button) {
                let mut mouse_button_input_events = self
                    .app
                    .resources
                    .get_mut::<Events<MouseButtonInput>>()
                    .unwrap();
                mouse_button_input_events.send(MouseButtonInput {
                    button: convert_mouse_button(button),
                    state: ElementState::Pressed,
                });
            }

            if input.mouse_button_released(button) {
                let mut mouse_button_input_events = self
                    .app
                    .resources
                    .get_mut::<Events<MouseButtonInput>>()
                    .unwrap();
                mouse_button_input_events.send(MouseButtonInput {
                    button: convert_mouse_button(button),
                    state: ElementState::Released,
                });
            }
        }

        let mouse_pos = input.mouse_pos();
        if self.last_mouse_pos != mouse_pos {
            let mut cursor_moved_events =
                self.app.resources.get_mut::<Events<CursorMoved>>().unwrap();
            cursor_moved_events.send(CursorMoved {
                id: WindowId::primary(),
                position: Vec2::new(mouse_pos.0, mouse_pos.1),
            });
        }

        self.app.update();

        None
    }

    fn render(&mut self, api: &mut dyn DoryenApi) {
        let render_function = {
            let fn_ref = self
                .app
                .resources
                .get::<RenderFn>()
                .expect("Cannot find render function resource");
            (*fn_ref).clone()
        };

        render_function(&mut self.app, api);
    }
}
