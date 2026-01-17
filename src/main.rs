use bevy::prelude::*;
use std::collections::HashMap;

const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const STARTING_VELOCITY: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const STARTING_ACCELERATION: Vec3 = Vec3::new(500.0, 500.0, 0.0);

#[derive(Component)]
struct Player(String);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Acceleration(Vec3);

#[derive(Component)]
struct FrictionForce;

fn friction(_t: &Transform, v: &Velocity) -> Acceleration {
    let acc_new = Acceleration(-0.01 * v.0.length_squared() * v.0.normalize_or(Vec3::new(0.0,0.0,0.0)));
    return acc_new;
}

fn old_friction(v: &Velocity) -> Acceleration {
    let acc_new = Acceleration(-0.01 * v.0.length_squared() * v.0.normalize_or(Vec3::new(0.0,0.0,0.0)));
    return acc_new;
}

fn player_accelerate_new(keyboard_input: Res<ButtonInput<KeyCode>>,
                         mut query: Query<(&mut Velocity, &Acceleration, &Player)>,
                         time: Res<Time>) {
    for (mut v, a, player) in &mut query {
        println!("Player: {}, time: {:?}", player.0, time);
    }
}

fn player_accelerate(keyboard_input: Res<ButtonInput<KeyCode>>,
               mut player_velocity_acc: Single<(&mut Velocity, &Acceleration), With<Player>>,
               time: Res<Time>) {
    let player_velocity = &player_velocity_acc.0;
    let player_acceleration = &player_velocity_acc.1;
    let mut direction = Vec3::new(0.0, 0.0, 0.0);

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    let new_player_velocity = player_velocity.0 + (direction * player_acceleration.0 +  old_friction(player_velocity).0) * time.delta_secs();
    player_velocity_acc.0.0 = new_player_velocity;
}

fn friction_force(mut query: Query<(&Transform, &Velocity, &mut Acceleration), (With<FrictionForce>)>) {
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
    }
}

fn initialize(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((Player(String::from("player1")),
                    Acceleration(STARTING_ACCELERATION.clone()),
                    Velocity(STARTING_VELOCITY.clone()),
                    Transform {
                        translation: Vec3::new(0.0, 25.0, 0.0),
                        scale: Vec2::new(50.0, 50.0).extend(1.0),
                        ..default()
                    },
                    Sprite::from_color(Color::srgb(1.0,0.0,0.0), Vec2::ONE)));
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOR));
        app.add_systems(Startup, initialize);
        app.add_systems(FixedUpdate, (player_accelerate_new, player_accelerate, apply_velocity).chain());
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .run();
}
