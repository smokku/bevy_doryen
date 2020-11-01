// This example is adapted from https://github.com/jice-nospam/doryen-rs/blob/master/examples/basic.rs

use bevy::{
    input::{keyboard::KeyCode, Input},
    prelude::*,
};
use bevy_doryen::{doryen::TextAlign, Doryen, DoryenConfig, DoryenPlugin};

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
        .add_resource(WindowDescriptor {
            title: "MyRoguelike".to_string(),
            width: CONSOLE_WIDTH * 8,
            height: CONSOLE_HEIGHT * 8,
            ..Default::default()
        })
        .add_default_plugins()
        .add_plugin(DoryenPlugin)
        .add_resource(DoryenConfig {
            console_width: CONSOLE_WIDTH,
            console_height: CONSOLE_HEIGHT,
            font: "terminal_8x8.png",
        })
        .add_resource(MyRoguelike::new())
        .add_startup_system(init.system())
        .add_system_to_stage(stage::PRE_UPDATE, input.system()) // game inputs
        .add_system_to_stage(stage::POST_UPDATE, render.system()) // render game state to console
        .run();
}

fn init(mut doryen: ResMut<Doryen>) {
    let con = doryen.con_mut();
    con.register_color("white", (255, 255, 255, 255));
    con.register_color("red", (255, 92, 92, 255));
    con.register_color("blue", (192, 192, 255, 255));
}

fn render(game: Res<MyRoguelike>, mut doryen: ResMut<Doryen>) {
    let con = doryen.con_mut();
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

fn input(
    keyboard_input: Res<Input<KeyCode>>,
    mut cursor_moved_reader: Local<EventReader<CursorMoved>>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    mut game: ResMut<MyRoguelike>,
    doryen: Res<Doryen>,
    windows: Res<Windows>,
) {
    if keyboard_input.pressed(KeyCode::Left) {
        game.player_pos.0 = (game.player_pos.0 - 1).max(1);
    } else if keyboard_input.pressed(KeyCode::Right) {
        game.player_pos.0 = (game.player_pos.0 + 1).min(CONSOLE_WIDTH as i32 - 2);
    }
    if keyboard_input.pressed(KeyCode::Up) {
        game.player_pos.1 = (game.player_pos.1 - 1).max(1);
    } else if keyboard_input.pressed(KeyCode::Down) {
        game.player_pos.1 = (game.player_pos.1 + 1).min(CONSOLE_HEIGHT as i32 - 2);
    }

    let window = windows.get_primary().unwrap();
    for event in cursor_moved_reader.iter(&cursor_moved_events) {
        game.mouse_pos = doryen.con().pixel_to_pos(
            event.position.x(),
            window.height() as f32 - event.position.y(),
            window.width(),
            window.height(),
        );
    }
}
