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
const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

use bevy::prelude::*;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component, Debug, Clone, Copy)] // Deref/DerefMut entfernt
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Debug, Clone, Copy)]
struct Size {
    width: f32,
    height: f32,
}

#[derive(Component, Debug, Clone, Copy)] // Deref/DerefMut entfernt
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

#[derive(Component)]
struct ScoreText;

fn main() {
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
        .insert_resource(Score { left: 0, right: 0 })
        .add_systems(Startup, setup_game)
        .add_systems(Update,
                     (
                         move_paddles_system,
                         (move_ball_system, collision_system, scoring_system).chain(),
                         sync_positions_to_transforms, // NEUES SYSTEM HINZUGEFÜGT
                         update_score_display_system,
                     )
        )
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn left paddle
    let left_paddle_x = -SCREEN_WIDTH / 2.0 + PADDLE_WALL_PADDING + PADDLE_WIDTH / 2.0;
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(left_paddle_x, 0.0, 0.0),
            sprite: Sprite {
                color: ENTITY_COLOR,
                custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
                ..default()
            },
            ..default()
        },
        Paddle,
        Position { x: left_paddle_x, y: 0.0 },
        Size { width: PADDLE_WIDTH, height: PADDLE_HEIGHT },
        MoveSpeed(PADDLE_SPEED),
    ));

    // Spawn right paddle
    let right_paddle_x = SCREEN_WIDTH / 2.0 - PADDLE_WALL_PADDING - PADDLE_WIDTH / 2.0;
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(right_paddle_x, 0.0, 0.0),
            sprite: Sprite {
                color: ENTITY_COLOR,
                custom_size: Some(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
                ..default()
            },
            ..default()
        },
        Paddle,
        Position { x: right_paddle_x, y: 0.0 },
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
                // KORRIGIERTER PFAD (angenommen, die Schriftart ist in assets/fonts/)
                font: asset_server.load("FiraMono-Medium.ttf"),
                font_size: 40.0,
                color: SCORE_TEXT_COLOR,
            },
        )
            .with_style(Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                // Geändert für bessere Sichtbarkeit/Debugging, zentriert über die gesamte Breite
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center, // Zentriert den Text horizontal
                // align_self: AlignSelf::Center, // Kann auch verwendet werden, aber JustifyContent auf dem Parent ist oft besser
                ..default()
            }),
        ScoreText,
    ));
}

// NEUES SYSTEM: Aktualisiert die Transform-Komponente basierend auf der Position-Komponente
fn sync_positions_to_transforms(
    mut query: Query<(&Position, &mut Transform), Changed<Position>>, // Nur Entitäten mit geänderter Position
) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x;
        transform.translation.y = position.y;
    }
}

fn move_paddles_system(
    // Wichtig: Da Position jetzt Copy ist, brauchen wir keine explizite Unterscheidung
    // für linkes/rechtes Paddle mehr im Query-Typ, können es aber in der Logik beibehalten.
    // Wir fragen jetzt `(&mut Position, &MoveSpeed, Entity)` um zu unterscheiden, welches Paddle es ist.
    // Alternativ könnte man Marker-Komponenten `LeftPaddle` und `RightPaddle` verwenden.
    // Für die Einfachheit belasse ich es bei der x-Koordinaten-Prüfung.
    mut paddle_query: Query<(&mut Position, &MoveSpeed, &Transform), With<Paddle>>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let delta_seconds = time.delta_seconds();

    for (mut position, move_speed, transform) in paddle_query.iter_mut() {
        let mut direction = 0.0;

        // Unterscheidung über die aktuelle x-Position der Transform Komponente
        if transform.translation.x < 0.0 { // Linkes Paddle
            if keyboard_input.pressed(KeyCode::W) { direction += 1.0; }
            if keyboard_input.pressed(KeyCode::S) { direction -= 1.0; }
        } else { // Rechtes Paddle
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
    paddle_query: Query<(&Position, &Size), With<Paddle>>, // Position ist hier nur lesend
) {
    if let Ok((mut ball_velocity, ball_pos, ball_size_component)) = ball_query.get_single_mut() {
        let ball_half_width = ball_size_component.width / 2.0; // Korrekter, da BALL_SIZE für width & height steht
        let ball_half_height = ball_size_component.height / 2.0;


        // Kollision mit oberen/unteren Wänden
        if (ball_pos.y + ball_half_height > SCREEN_HEIGHT / 2.0 && ball_velocity.y > 0.0) ||
            (ball_pos.y - ball_half_height < -SCREEN_HEIGHT / 2.0 && ball_velocity.y < 0.0) {
            ball_velocity.y *= -1.0;
        }

        // Kollision mit Paddles
        for (paddle_pos, paddle_size) in paddle_query.iter() {
            let ball_left = ball_pos.x - ball_half_width;
            let ball_right = ball_pos.x + ball_half_width;
            let ball_top = ball_pos.y + ball_half_height;
            let ball_bottom = ball_pos.y - ball_half_height;

            let paddle_left = paddle_pos.x - paddle_size.width / 2.0;
            let paddle_right = paddle_pos.x + paddle_size.width / 2.0;
            let paddle_top = paddle_pos.y + paddle_size.height / 2.0;
            let paddle_bottom = paddle_pos.y - paddle_size.height / 2.0;

            // AABB Kollisionscheck
            if ball_left < paddle_right && ball_right > paddle_left &&
                ball_bottom < paddle_top && ball_top > paddle_bottom {
                // Einfache Überprüfung, ob der Ball sich auf das Paddle zubewegt, um Mehrfachkollisionen zu vermeiden
                // wenn der Ball schon im Paddle "steckt"
                let moving_towards_left_paddle = ball_velocity.x < 0.0 && paddle_pos.x < 0.0; // Ball nach links, linkes Paddle
                let moving_towards_right_paddle = ball_velocity.x > 0.0 && paddle_pos.x > 0.0; // Ball nach rechts, rechtes Paddle

                if moving_towards_left_paddle || moving_towards_right_paddle {
                    ball_velocity.x *= -1.0;
                    let offset_y = (ball_pos.y - paddle_pos.y) / (paddle_size.height / 2.0);
                    let clamped_offset_y = offset_y.clamp(-1.0, 1.0);
                    // Die y-Geschwindigkeit etwas weniger stark vom x-Wert abhängig machen, um extreme Winkel zu vermeiden
                    ball_velocity.y = clamped_offset_y * INITIAL_BALL_SPEED_X * 0.75;
                    ball_velocity.x *= BALL_SPEED_INCREASE_FACTOR;
                    // Die y-Geschwindigkeit auch erhöhen, aber vielleicht nicht so stark, oder relativ zur neuen x-Geschwindigkeit
                    ball_velocity.y *= BALL_SPEED_INCREASE_FACTOR.sqrt(); // Etwas sanftere Erhöhung für y
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
        let ball_half_width = ball_size_component.width / 2.0;
        let mut scored = false;

        // Rechter Spieler punktet (Ball geht links raus)
        if ball_pos.x + ball_half_width < -SCREEN_WIDTH / 2.0 {
            score.right += 1;
            ball_velocity.x = INITIAL_BALL_SPEED_X; // Ball bewegt sich nach rechts
            ball_velocity.y = INITIAL_BALL_SPEED_Y; // oder ein kleiner zufälliger Y-Wert
            scored = true;
        }
        // Linker Spieler punktet (Ball geht rechts raus)
        else if ball_pos.x - ball_half_width > SCREEN_WIDTH / 2.0 {
            score.left += 1;
            ball_velocity.x = -INITIAL_BALL_SPEED_X; // Ball bewegt sich nach links
            ball_velocity.y = INITIAL_BALL_SPEED_Y; // oder ein kleiner zufälliger Y-Wert
            scored = true;
        }

        if scored {
            println!("Score: Left {} - Right {}", score.left, score.right);
            ball_pos.x = 0.0;
            ball_pos.y = 0.0;
            // Optional: Geschwindigkeit nach Punkt zurücksetzen oder leicht reduzieren
            // ball_velocity.x = if ball_velocity.x > 0.0 { INITIAL_BALL_SPEED_X } else { -INITIAL_BALL_SPEED_X };
            // ball_velocity.y = INITIAL_BALL_SPEED_Y;
        }
    }
}

fn update_score_display_system(
    score: Res<Score>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    // .is_changed() ist gut, aber bei UI kann es auch Sinn machen, immer zu aktualisieren,
    // falls andere Dinge (wie Fenstergröße, falls dynamisch) den Text beeinflussen könnten.
    // Für den Score reicht .is_changed() aber aus.
    if score.is_changed() {
        for mut text in query.iter_mut() {
            // Sicherstellen, dass es mindestens eine Sektion gibt
            if let Some(first_section) = text.sections.get_mut(0) {
                first_section.value = format!("Left: {}  Right: {}", score.left, score.right);
            }
        }
    }
}