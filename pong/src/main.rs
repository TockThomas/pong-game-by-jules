use bevy::prelude::Color;

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const PADDLE_WIDTH: f32 = 20.0;
const PADDLE_HEIGHT: f32 = 100.0;
const PADDLE_SPEED: f32 = 500.0;
const BALL_SIZE: f32 = 15.0;
const INITIAL_BALL_SPEED_X: f32 = 200.0;
const INITIAL_BALL_SPEED_Y: f32 = 0.0;
const BALL_SPEED_INCREASE_FACTOR: f32 = 1.05;
const PADDLE_WALL_PADDING: f32 = 10.0;
const SCORE_TEXT_COLOR: Color = Color::WHITE;
const ENTITY_COLOR: Color = Color::WHITE;

// Basic main function to make the file compile
fn main() {
    // The rest of the application logic will be added in later subtasks.
    println!("Constants loaded. Welcome to Pong!");
}

use bevy::prelude::*;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component, Deref, DerefMut)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

#[derive(Component, Deref, DerefMut)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct MoveSpeed(f32);

#[derive(Resource)]
struct Score {
    left: u32,
    right: u32,
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Insert Score resource
    commands.insert_resource(Score { left: 0, right: 0 });

    // Spawn left paddle
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(-SCREEN_WIDTH / 2.0 + PADDLE_WALL_PADDING + PADDLE_WIDTH / 2.0, 0.0, 0.0),
            sprite: Sprite {
                color: ENTITY_COLOR,
                custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
                ..default()
            },
            ..default()
        },
        Paddle,
        Position { x: -SCREEN_WIDTH / 2.0 + PADDLE_WALL_PADDING + PADDLE_WIDTH / 2.0, y: 0.0 },
        Size { width: PADDLE_WIDTH, height: PADDLE_HEIGHT },
        MoveSpeed(PADDLE_SPEED),
    ));

    // Spawn right paddle
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(SCREEN_WIDTH / 2.0 - PADDLE_WALL_PADDING - PADDLE_WIDTH / 2.0, 0.0, 0.0),
            sprite: Sprite {
                color: ENTITY_COLOR,
                custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
                ..default()
            },
            ..default()
        },
        Paddle,
        Position { x: SCREEN_WIDTH / 2.0 - PADDLE_WALL_PADDING - PADDLE_WIDTH / 2.0, y: 0.0 },
        Size { width: PADDLE_WIDTH, height: PADDLE_HEIGHT },
        MoveSpeed(PADDLE_SPEED),
    ));

    // Spawn ball
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            sprite: Sprite {
                color: ENTITY_COLOR,
                custom_size: Some(Vec2::new(BALL_SIZE, BALL_SIZE)),
                ..default()
            },
            ..default()
        },
        Ball,
        Position { x: 0.0, y: 0.0 },
        Size { width: BALL_SIZE, height: BALL_SIZE },
        Velocity { x: INITIAL_BALL_SPEED_X, y: INITIAL_BALL_SPEED_Y },
    ));

    // Spawn score text
    commands.spawn(TextBundle::from_section(
        "Left: 0  Right: 0",
        TextStyle {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"), // Using a default Bevy font path
            font_size: 40.0,
            color: SCORE_TEXT_COLOR,
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        align_self: AlignSelf::Center,
        ..default()
    }));
}

fn move_paddles_system(
    mut paddle_query: Query<(&mut Position, &MoveSpeed), With<Paddle>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let delta_seconds = time.delta_seconds();

    for (mut position, move_speed) in paddle_query.iter_mut() {
        let mut direction = 0.0;

        // Left paddle (negative x position)
        if position.x < 0.0 {
            if keyboard_input.pressed(KeyCode::W) {
                direction += 1.0;
            }
            if keyboard_input.pressed(KeyCode::S) {
                direction -= 1.0;
            }
        }
        // Right paddle (positive x position)
        else {
            if keyboard_input.pressed(KeyCode::Up) {
                direction += 1.0;
            }
            if keyboard_input.pressed(KeyCode::Down) {
                direction -= 1.0;
            }
        }

        let new_y = position.y + direction * move_speed.0 * delta_seconds;

        // Clamp paddle position - assuming (0,0) is screen center
        let upper_bound = SCREEN_HEIGHT / 2.0 - PADDLE_HEIGHT / 2.0;
        let lower_bound = -SCREEN_HEIGHT / 2.0 + PADDLE_HEIGHT / 2.0;
        position.y = new_y.clamp(lower_bound, upper_bound);
    }
}

fn move_ball_system(
    mut ball_query: Query<(&mut Position, &Velocity), With<Ball>>,
    time: Res<Time>,
) {
    if let Ok((mut position, velocity)) = ball_query.get_single_mut() {
        let delta_seconds = time.delta_seconds();
        position.x += velocity.x * delta_seconds;
        position.y += velocity.y * delta_seconds;
    }
}

fn collision_system(
    mut ball_query: Query<(&mut Velocity, &Position, &Size), With<Ball>>,
    paddle_query: Query<(&Position, &Size), With<Paddle>>,
) {
    if let Ok((mut ball_velocity, ball_pos, ball_size_component)) = ball_query.get_single_mut() {
        let ball_half_size = ball_size_component.width / 2.0; // Assuming ball is square

        // Wall collision (top/bottom) - assuming (0,0) is screen center
        if (ball_pos.y + ball_half_size > SCREEN_HEIGHT / 2.0 && ball_velocity.y > 0.0) ||
           (ball_pos.y - ball_half_size < -SCREEN_HEIGHT / 2.0 && ball_velocity.y < 0.0) {
            ball_velocity.y *= -1.0;
        }

        // Paddle collision
        for (paddle_pos, paddle_size) in paddle_query.iter() {
            let ball_left = ball_pos.x - ball_half_size;
            let ball_right = ball_pos.x + ball_half_size;
            let ball_top = ball_pos.y + ball_half_size;
            let ball_bottom = ball_pos.y - ball_half_size;

            let paddle_left = paddle_pos.x - paddle_size.width / 2.0;
            let paddle_right = paddle_pos.x + paddle_size.width / 2.0;
            let paddle_top = paddle_pos.y + paddle_size.height / 2.0;
            let paddle_bottom = paddle_pos.y - paddle_size.height / 2.0;

            // AABB collision check
            if ball_left < paddle_right && ball_right > paddle_left &&
               ball_bottom < paddle_top && ball_top > paddle_bottom {

                // Collision occurred
                // Reverse x velocity only if ball is moving towards the paddle
                if (ball_velocity.x > 0.0 && ball_pos.x < paddle_pos.x) || // Ball moving right, hits left side of right paddle
                   (ball_velocity.x < 0.0 && ball_pos.x > paddle_pos.x) {  // Ball moving left, hits right side of left paddle
                    ball_velocity.x *= -1.0;

                    // Adjust y velocity based on impact point
                    let offset_y = (ball_pos.y - paddle_pos.y) / (paddle_size.height / 2.0);
                    let clamped_offset_y = offset_y.clamp(-1.0, 1.0); // Clamp between -1.0 and 1.0

                    // The y-velocity adjustment depends on the new x-velocity's magnitude
                    // and the offset. A common factor is applied.
                    ball_velocity.y = clamped_offset_y * ball_velocity.x.abs() * 0.75;

                    // Increase ball speed
                    ball_velocity.x *= BALL_SPEED_INCREASE_FACTOR;
                    ball_velocity.y *= BALL_SPEED_INCREASE_FACTOR;
                }
            }
        }
    }
}

fn scoring_system(
    mut ball_query: Query<(&mut Position, &mut Velocity, &Size), With<Ball>>,
    mut score: ResMut<Score>,
) {
    if let Ok((mut ball_pos, mut ball_velocity, ball_size_component)) = ball_query.get_single_mut() {
        let ball_half_size = ball_size_component.width / 2.0; // Assuming ball is square

        let mut scored = false;

        // Right player scores (ball passes left edge)
        if ball_pos.x + ball_half_size < -SCREEN_WIDTH / 2.0 {
            score.right += 1;
            println!("Score: Left {} - Right {}", score.left, score.right);
            ball_velocity.x = INITIAL_BALL_SPEED_X; // Serve to right player (who just scored)
            ball_velocity.y = INITIAL_BALL_SPEED_Y;
            scored = true;
        }
        // Left player scores (ball passes right edge)
        else if ball_pos.x - ball_half_size > SCREEN_WIDTH / 2.0 {
            score.left += 1;
            println!("Score: Left {} - Right {}", score.left, score.right);
            ball_velocity.x = -INITIAL_BALL_SPEED_X; // Serve to left player (who just scored)
            ball_velocity.y = INITIAL_BALL_SPEED_Y;
            scored = true;
        }

        if scored {
            ball_pos.x = 0.0;
            ball_pos.y = 0.0;
            // Note: Ball speed is reset to initial, not increased further by BALL_SPEED_INCREASE_FACTOR here.
            // That factor applies on paddle hits.
        }
    }
}
