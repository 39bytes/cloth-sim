use cloth_sim::math::Vector2;
use cloth_sim::Cloth;
use notan::draw::*;
use notan::prelude::*;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const CLOTH_WIDTH: i32 = 20;
const CLOTH_HEIGHT: i32 = 20;
const CLOTH_SPACING: i32 = 10;

fn main() -> Result<(), String> {
    let win_config = WindowConfig::new().size(WIDTH, HEIGHT).vsync(true);

    notan::init_with(setup)
        .add_config(win_config)
        .draw(draw)
        .update(update)
        .add_config(DrawConfig)
        .build()
}

#[derive(AppState)]
struct State {
    cloths: Vec<Cloth>,
    prev_mouse_position: Vector2,
}

fn setup() -> State {
    // Initialize the state
    // Instantiate cloths here
    let cloths = vec![
        Cloth::new(
            CLOTH_WIDTH,
            CLOTH_HEIGHT,
            CLOTH_SPACING,
            WIDTH / 2 - CLOTH_WIDTH * CLOTH_SPACING,
            HEIGHT / 10,
            10.0,
        ),
        Cloth::new(
            CLOTH_WIDTH,
            CLOTH_HEIGHT,
            CLOTH_SPACING,
            WIDTH / 2 - CLOTH_WIDTH * CLOTH_SPACING + 200,
            HEIGHT / 10,
            10.0,
        ),
    ];
    State {
        cloths,
        prev_mouse_position: Vector2::ZERO,
    }
}

fn update(app: &mut App, state: &mut State) {
    let dt = app.timer.delta().as_secs_f64();
    for cloth in &mut state.cloths {
        cloth.update(dt, &app.mouse, state.prev_mouse_position);
    }

    state.prev_mouse_position = Vector2::from(app.mouse.position());
}

fn draw(gfx: &mut Graphics, state: &mut State) {
    let mut draw = gfx.create_draw();

    draw.clear(Color::BLACK);

    for cloth in &mut state.cloths {
        cloth.draw(&mut draw);
    }
    gfx.render(&draw);
}
