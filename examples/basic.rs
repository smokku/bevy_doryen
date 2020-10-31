// This example is adapted from https://github.com/jice-nospam/doryen-rs/blob/master/examples/basic.rs

use bevy::{
    input::keyboard::{ElementState, KeyboardInput},
    prelude::*,
};
use bevy_doryen::{doryen::TextAlign, AppOptions, DoryenApi, DoryenPlugin, InitFn, RenderFn};
use std::sync::Arc;

const CONSOLE_WIDTH: u32 = 80;
const CONSOLE_HEIGHT: u32 = 45;

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
        // plugin stuff
        .add_resource(AppOptions {
            console_width: CONSOLE_WIDTH,
            console_height: CONSOLE_HEIGHT,
            screen_width: CONSOLE_WIDTH * 8,
            screen_height: CONSOLE_HEIGHT * 8,
            window_title: "my roguelike".to_owned(),
            font_path: "terminal_8x8.png".to_owned(),
            vsync: true,
            fullscreen: false,
            show_cursor: true,
            resizable: true,
            intercept_close_request: false,
            max_fps: 0,
        })
        .add_resource::<InitFn>(Arc::new(Box::new(init)))
        .add_resource::<RenderFn>(Arc::new(Box::new(render)))
        .add_plugin(DoryenPlugin)
        // example stuff
        .add_resource(MyRoguelike::new())
        .add_system(input.system())
        .run();
}

fn init(api: &mut dyn DoryenApi, _app: &mut App) {
    api.con().register_color("white", (255, 255, 255, 255));
    api.con().register_color("red", (255, 92, 92, 255));
    api.con().register_color("blue", (192, 192, 255, 255));
}

fn render(app: &mut App, api: &mut dyn DoryenApi) {
    let game = app.resources.get::<MyRoguelike>().unwrap();

    let con = api.con();
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
