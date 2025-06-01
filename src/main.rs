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
const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1); // Added background color

use bevy::prelude::*; // Keep this for general prelude items

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component, Debug, Copy, Clone, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
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

// Marker component for the score text UI entity
#[derive(Component)]
struct ScoreText;

fn update_sprite_transforms_from_position(
    mut query: Query<(&mut Transform, &Position)>,
) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        // transform.translation.z is not managed by Position, so leave it as is.
    }
}

fn main() {
    use bevy::prelude::*;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pong".into(),
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Score { left: 0, right: 0 }) // Score resource initialized here
        .add_systems(Startup, setup_game)
        .add_systems(Update,
            (
                move_paddles_system,
                move_ball_system,
                update_sprite_transforms_from_position.after(move_paddles_system).after(move_ball_system),
                collision_system.after(update_sprite_transforms_from_position),
                scoring_system.after(collision_system),
                update_score_display_system.after(scoring_system),
            )
        )
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // commands.insert_resource(Score { left: 0, right: 0 }); // Removed from here

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
    commands.spawn((
        TextBundle::from_section(
            "Left: 0  Right: 0",
            TextStyle {
                font: asset_server.load("FiraMono-Medium.ttf"),
                font_size: 40.0,
                color: SCORE_TEXT_COLOR,
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        ScoreText,
    ));
}

fn move_paddles_system(
    mut paddle_query: Query<(&mut Position, &MoveSpeed), With<Paddle>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let delta_seconds = time.delta_seconds();

    for (mut position, move_speed) in paddle_query.iter_mut() {
        let mut direction = 0.0;

        if position.x < 0.0 { // Left paddle
            if keyboard_input.pressed(KeyCode::W) { direction += 1.0; }
            if keyboard_input.pressed(KeyCode::S) { direction -= 1.0; }
        } else { // Right paddle
            if keyboard_input.pressed(KeyCode::Up) { direction += 1.0; }
            if keyboard_input.pressed(KeyCode::Down) { direction -= 1.0; }
        }

        let new_y = position.y + direction * move_speed.0 * delta_seconds;
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
        position.x += velocity.x * time.delta_seconds();
        position.y += velocity.y * time.delta_seconds();
    }
}

fn collision_system(
    mut ball_query: Query<(&mut Velocity, &Position, &Size), With<Ball>>,
    paddle_query: Query<(&Position, &Size), With<Paddle>>,
) {
    if let Ok((mut ball_velocity, ball_pos, ball_size_component)) = ball_query.get_single_mut() {
        let ball_half_size = ball_size_component.width / 2.0;

        if (ball_pos.y + ball_half_size > SCREEN_HEIGHT / 2.0 && ball_velocity.y > 0.0) ||
           (ball_pos.y - ball_half_size < -SCREEN_HEIGHT / 2.0 && ball_velocity.y < 0.0) {
            ball_velocity.y *= -1.0;
        }

        for (paddle_pos, paddle_size) in paddle_query.iter() {
            let ball_left = ball_pos.x - ball_half_size;
            let ball_right = ball_pos.x + ball_half_size;
            let ball_top = ball_pos.y + ball_half_size;
            let ball_bottom = ball_pos.y - ball_half_size;

            let paddle_left = paddle_pos.x - paddle_size.width / 2.0;
            let paddle_right = paddle_pos.x + paddle_size.width / 2.0;
            let paddle_top = paddle_pos.y + paddle_size.height / 2.0;
            let paddle_bottom = paddle_pos.y - paddle_size.height / 2.0;

            if ball_left < paddle_right && ball_right > paddle_left &&
               ball_bottom < paddle_top && ball_top > paddle_bottom {
                if (ball_velocity.x > 0.0 && ball_pos.x < paddle_pos.x) ||
                   (ball_velocity.x < 0.0 && ball_pos.x > paddle_pos.x) {
                    ball_velocity.x *= -1.0;
                    let offset_y = (ball_pos.y - paddle_pos.y) / (paddle_size.height / 2.0);
                    let clamped_offset_y = offset_y.clamp(-1.0, 1.0);
                    ball_velocity.y = clamped_offset_y * ball_velocity.x.abs() * 0.75;
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
        let ball_half_size = ball_size_component.width / 2.0;
        let mut scored = false;

        if ball_pos.x + ball_half_size < -SCREEN_WIDTH / 2.0 { // Right player scores
            score.right += 1;
            ball_velocity.x = INITIAL_BALL_SPEED_X;
            ball_velocity.y = INITIAL_BALL_SPEED_Y;
            scored = true;
        } else if ball_pos.x - ball_half_size > SCREEN_WIDTH / 2.0 { // Left player scores
            score.left += 1;
            ball_velocity.x = -INITIAL_BALL_SPEED_X;
            ball_velocity.y = INITIAL_BALL_SPEED_Y;
            scored = true;
        }

        if scored {
            println!("Score: Left {} - Right {}", score.left, score.right);
            ball_pos.x = 0.0;
            ball_pos.y = 0.0;
        }
    }
}

fn update_score_display_system(
    score: Res<Score>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if score.is_changed() {
        for mut text in query.iter_mut() {
            if !text.sections.is_empty() {
                text.sections[0].value = format!("Left: {}  Right: {}", score.left, score.right);
            }
        }
    }
}
