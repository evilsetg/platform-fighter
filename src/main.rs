use bevy::prelude::*;
use bevy::{
    math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
};
use bevy::{audio::Volume};
use bevy::text::LineBreak;
use std::mem::swap;
use std::time::Duration;

const NULL_VECTOR: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const STARTING_VELOCITY: Vec3 = NULL_VECTOR;
const STARTING_ACCELERATION: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const PLAYER_MOVEMENT_FORCE: Vec3 = Vec3::new(2.0, 0.0, 0.0);
const PLAYER_MOVEMENT_FORCE_AIR: Vec3 = Vec3::new(1.5, 0.0, 0.0);
const GRAVTITON_FORCE: Vec3 = Vec3::new(0., -1., 0.);
const PLAYER_JUMP_VEL: f32 = 40.;
const SPECIAL_MOVE_MASS: f32 = 4.0;

#[derive(Component)]
struct Player(u32);

#[derive(Component)]
struct PlayerResult{
    player: u32,
    score: u32,
}

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct MovementForce{
    ground: Vec3,
    air: Vec3
}

#[derive(Component)]
struct Acceleration(Vec3);

#[derive(Component)]
struct FrictionForce;

#[derive(Component)]
struct Platform;

#[derive(Component)]
struct OnPlatform(bool);

#[derive(Component)]
struct GravitationForce(Vec3);

#[derive(Component)]
struct Score(u32);

#[derive(Component)]
struct ScoreDisplay(u32);

#[derive(Component)]
struct Mass(f32);

#[derive(Component)]
struct Cooldown {
    timer: Timer,
    charge: bool
}

#[derive(Component)]
struct GameOverText;

#[derive(Event)]
struct RespawnEvent {
    player: u32,
    score: u32
}

#[derive (Bundle)]
struct PlayerBundle {
    player: Player,
    acceleration: Acceleration,
    velocity: Velocity,
    mass: Mass,
    force_movement: MovementForce,
    force_friction: FrictionForce,
    force_gravitation: GravitationForce,
    on_platform: OnPlatform,
    special_move_cooldown: Cooldown,
    score: Score,
    transform: Transform,
    sprite: Sprite
}

#[derive (States, Debug, Clone, PartialEq, Eq, Hash)]
enum GameStates {
    Menu,
    Game,
    GameOver,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct MenuSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct GameSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct GameOverSet;

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player(1),
            acceleration: Acceleration(NULL_VECTOR.clone()),
            velocity: Velocity(NULL_VECTOR.clone()),
            mass: Mass(1.0),
            force_movement: MovementForce {
                ground: PLAYER_MOVEMENT_FORCE.clone(),
                air: PLAYER_MOVEMENT_FORCE_AIR.clone()
            },
            force_friction: FrictionForce,
            force_gravitation: GravitationForce(GRAVTITON_FORCE.clone()),
            on_platform: OnPlatform(false),
            special_move_cooldown: Cooldown {
                timer: Timer::from_seconds(2.0, TimerMode::Once),
                charge: true
            },
            score: Score(0),
            sprite: Default::default(),
            transform: Default::default()
        }
    }
}

fn jump(keyboard_input: Res<ButtonInput<KeyCode>>,
        mut query1: Query<(&mut Velocity, &Player, &OnPlatform)>,
        query2: Query<&Transform, With<Platform>>) {
    for (mut v, player, onPlatform) in &mut query1 {
        match player.0 {
            1 => {
                if keyboard_input.just_pressed(KeyCode::ArrowUp) && onPlatform.0 {
                    v.0.y += PLAYER_JUMP_VEL;
                }
            }
            2 => {
                if keyboard_input.just_pressed(KeyCode::KeyW) && onPlatform.0 {
                    v.0.y += PLAYER_JUMP_VEL;
                }
            }
            _ => {}
        }
    }
}

fn get_movement(player: &Player, keyboard_input: &Res<ButtonInput<KeyCode>>) -> Vec3 {
    let mut direction = NULL_VECTOR.clone();
    match player.0  {
        1 => {
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
        }
        2 => {
            if keyboard_input.pressed(KeyCode::KeyA) {
                direction.x -= 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyD) {
                direction.x += 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyW) {
                direction.y += 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyS) {
                direction.y -= 1.0;
            }
        }
        _ => {}
    }
    return direction.normalize_or(NULL_VECTOR);
}

fn movement_force(keyboard_input: Res<ButtonInput<KeyCode>>,
                  mut query: Query<(&mut Acceleration, &MovementForce,
                                    &Player, &OnPlatform)>) {
    for (mut accel, mf_accel, player, on_platform) in &mut query {
        
        let direction = get_movement(&player, &keyboard_input);

        if on_platform.0 {
            accel.0 += direction.normalize_or(NULL_VECTOR) * mf_accel.ground;
        }
        else {
            accel.0 += direction.normalize_or(NULL_VECTOR) * mf_accel.air;
        }
    }
}

fn special_move(
    mut query: Query<(&mut Velocity,
                      &mut Cooldown,
                      &mut Mass,
                      &mut Sprite,
                      &Player)>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>
) {
    for (mut v, mut cooldown, mut mass, mut sprite, player) in &mut query {
        if !cooldown.charge {
            cooldown.timer.tick(time.delta());
        }
        if cooldown.timer.elapsed() == Duration::from_secs_f32(0.5) {
            mass.0 = 1.0;
        }
        if cooldown.timer.is_finished() {
            cooldown.charge = true;
            println!("cooldown charge restored");
            cooldown.timer.reset();
            sprite.color = Color::srgb(1.0, 1.0, 1.0);
            
        }
        if cooldown.charge && match player.0 {
            1 => { keyboard_input.pressed(KeyCode::ShiftRight) }
            2 => { keyboard_input.pressed(KeyCode::ShiftLeft) }
            _ => { false }
        } {
            println!("player {} special move!", player.0);
            let mut direction = get_movement(&player, &keyboard_input);
            if direction.x == 0. && direction.y == 0.0 {
                v.0 = v.0 + 50. * v.0.normalize_or(NULL_VECTOR);
            }
            else {
                v.0 = v.0 + 50. * direction.normalize_or(NULL_VECTOR); 
            }
            cooldown.charge = false;
            sprite.color = Color::srgb(1.0, 0.7, 0.7);
            mass.0 = SPECIAL_MOVE_MASS;
        }
    }
}

fn friction_force(mut query: Query<(&Velocity, &mut Acceleration), With<FrictionForce>>) {
    for (v, mut accel) in &mut query {
        accel.0 += -(0.005 * v.0.length_squared() + 0.05 * v.0.length()) * v.0.normalize_or(NULL_VECTOR);
    }
}


fn gravitation_force(time: Res<Time>,
                     mut query: Query<(&mut Acceleration, &GravitationForce)>) {
    for (mut accel, g) in &mut query {
        accel.0 += g.0;
    }
}

fn respawn(mut query: Query<(&mut Score, &mut Transform, &Player)>,
           mut commands: Commands) {
    for (mut score, mut tf, player) in &mut query {
        if tf.translation.y < -800. {
            score.0 +=1;
            println!("Player {} respawn!, new score: {}", player.0, score.0);
            tf.translation.x = 0.;
            tf.translation.y = 25.;
            commands.trigger(RespawnEvent {
                player: player.0,
                score: score.0
            });
        }
    }
}


fn check_game_over(event: On<RespawnEvent>,
                   mut commands: Commands,
                   query: Query<(&Player, &Score)>,
                   mut next_state: ResMut<NextState<GameStates>>
) {
    if event.score > 5 {
        next_state.set(GameStates::GameOver);
        for (player, score) in &query {
            commands.spawn((
                DespawnOnExit(GameStates::GameOver),
                PlayerResult {
                    player: player.0,
                    score: score.0
                }));
        }
    }
}

fn show_score(event: On<RespawnEvent>,
              mut query1: Query<(&mut Text, &ScoreDisplay)>) {
    for (mut text, score_display) in &mut query1 {
        if score_display.0 == event.player {
            **text = event.score.to_string();
        }
    }
}

fn flip_sprite(mut query: Query<(&mut Sprite, &Velocity), With<Player>>) {
    for (mut sprite, v) in &mut query {
        if v.0.x.abs() >= 5. {
            match v.0.x > 0. {
                true => { sprite.flip_x = true; }
                false => { sprite.flip_x = false; }
            }
        }
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &mut Velocity, &mut Acceleration)>) {
    for (mut transform, mut velocity, mut accel) in &mut query {
        velocity.0 += accel.0;
        transform.translation.x += velocity.0.x;
        transform.translation.y += velocity.0.y;
        accel.0 = NULL_VECTOR.clone();
    }
}

fn platform_collide(mut query1: Query<(&mut Transform, &mut Velocity, &mut OnPlatform), Without<Platform>>,
           mut query2: Query<&Transform, With<Platform>>) {
    for (mut tf1, mut v1, mut on_platform) in &mut query1 {
        let mut is_on_platform = false;
        for (mut tf2) in &query2 {
            let bb1 = Aabb2d::new(
                tf1.translation.truncate(),
                tf1.scale.truncate() / 2.
            );
            let bb1_before = Aabb2d::new(
                (tf1.translation - v1.0).truncate(),
                tf1.scale.truncate() / 2.
            );
            let bb2 = Aabb2d::new(
                tf2.translation.truncate(),
                tf2.scale.truncate() / 2.
            );

            let offset = bb1_before.closest_point(bb2.center()) - bb2.center();
            let left =  offset.x <= 0.;
            let upper = offset.y >= 0.;
            let bb1_corner: Vec2;
            let bb2_corner: Vec2;
            let top_bottom: bool;
            if bb1.intersects(&bb2) {
                /*let collision =*/ match (left, upper) {
                    (false, false) => {
                        // lower right
                        top_bottom = false;
                        // take upper left corner of bb1
                        bb1_corner = Vec2::new(bb1.min.x, bb1.max.y);
                        // take lower right corner of bb2
                        bb2_corner = Vec2::new(bb2.max.x, bb2.min.y);
                    }
                    (true, false) => {
                        // lower left
                        top_bottom = false;
                        // take upper right corner of bb1
                        bb1_corner = bb1.max;
                        // take bottom left corner of bb2
                        bb2_corner = bb2.min;
                    }
                    (false, true) => {
                        // upper right
                        top_bottom = true;
                        // take bottom left corner of bb1
                        bb1_corner = bb1.min;
                        // take upper right corner of bb2
                        bb2_corner = bb2.max;
                    }
                    (true, true) => {
                        // upper left
                        top_bottom = true;
                        // take bottom right corner of bb1
                        bb1_corner = Vec2::new(bb1.max.x, bb1.min.y);
                        // take upper left corner of bb2
                        bb2_corner = Vec2::new(bb2.min.x, bb2.max.y);
                    }
                }
                let bb_distance = bb2_corner - bb1_corner;
                let time_of_collision = - bb_distance / v1.0.xy();
                // println!("bb_distance: {:?}", bb_distance);
                // println!("v1: {:?}", v1.0);
                // println!("toc: {}", time_of_collision);
                if time_of_collision.x > 0. && time_of_collision.x < time_of_collision.y.abs() {
                    // println!("x collision");
                    tf1.translation.x += bb_distance.x;
                    v1.0.x = 0.;
                }
                else {
                    // println!("y collision");
                    tf1.translation.y += bb_distance.y;
                    v1.0.y = 0.;
                    if top_bottom {
                        is_on_platform = true;
                    }
                }
            }
        }
        on_platform.0 = is_on_platform;
    }
}

fn rematch(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameStates>>, 
    mut query_scoreboard: Query<&mut Text, With<ScoreDisplay>>,
    keyboard_input: Res<ButtonInput<KeyCode>>
) {
    if keyboard_input.pressed(KeyCode::KeyR) {
        for mut score_text in &mut query_scoreboard {
            **score_text = "0".to_string();
        }
        next_state.set(GameStates::Game);
    }
}

fn game_over_screen(
    query_player_results: Query<(&PlayerResult)>,
    mut commands: Commands, mut asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/terminal-grotesque.ttf");
    let soundtrack = asset_server.load::<AudioSource>("sounds/game_over.ogg");
    let mut min_points = 6;
    let mut player_winner = 0;
    for player_result in &query_player_results {
        if player_result.score < min_points {
            player_winner = player_result.player;
            min_points = player_result.score;
        }
    }
    let winner_string = match player_winner {
        1 => { "Penguin" }
        2 => { "Seal" }
        _ => { "???" }
    };
    commands.spawn((
        DespawnOnExit(GameStates::GameOver),
        GameOverText,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            height: Val::Px(400.),
            width: percent(100),
            ..default()
        },
        Text(format!("{} wins!\n [R]ematch?", winner_string).to_string()),
        TextLayout::new(Justify::Center, LineBreak::AnyCharacter),
        TextColor(Color::BLACK),
        TextFont {
            font: font,
            font_size: 128.,
            ..default()
        },
    ));
    commands.spawn((
        DespawnOnExit(GameStates::GameOver),
        AudioPlayer(soundtrack.clone()),
        PlaybackSettings::ONCE,
    ));
}

fn player_collide(mut query: Query<(&mut Transform, &mut Velocity, &mut OnPlatform, &Player, &Mass)>) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([(mut tf1, mut v1, mut jump_charge1, player1, m1),
                    (mut tf2, mut v2, mut jump_charge2, player2, m2)]) = combinations.fetch_next() {
        let bb1 = Aabb2d::new(
            tf1.translation.truncate(),
            tf1.scale.truncate() / 2.
        );

        let bb2 = Aabb2d::new(
            tf2.translation.truncate(),
            tf2.scale.truncate() / 2.
        );

        let v2_trunc = v2.0.truncate();
        let offset = ((tf1.translation - v1.0) - (tf2.translation - v2.0)).truncate();
        let left =  offset.x <= 0.;
        let upper = offset.y >= 0.;
        let bb1_corner: Vec2;
        let bb2_corner: Vec2;
        if bb1.intersects(&bb2) {
            let top_bottom: bool;
            /*let collision =*/ match (left, upper) {
                (false, false) => {
                    top_bottom = false;
                    // lower right
                    // take upper left corner of bb1
                    bb1_corner = Vec2::new(bb1.min.x, bb1.max.y);
                    // take lower right corner of bb2
                    bb2_corner = Vec2::new(bb2.max.x, bb2.min.y);
                }
                (true, false) => {
                    top_bottom = false;
                    // lower left
                    // take upper right corner of bb1
                    bb1_corner = bb1.max;
                    // take bottom left corner of bb2
                    bb2_corner = bb2.min;
                }
                (false, true) => {
                    top_bottom = true;
                    // upper right
                    // take bottom left corner of bb1
                    bb1_corner = bb1.min;
                    // take upper right corner of bb2
                    bb2_corner = bb2.max;
                }
                (true, true) => {
                    top_bottom = true;
                    // upper left
                    // take bottom right corner of bb1
                    bb1_corner = Vec2::new(bb1.max.x, bb1.min.y);
                    // take upper left corner of bb2
                    bb2_corner = Vec2::new(bb2.min.x, bb2.max.y);
                }
            }
            let bb_distance = bb2_corner - bb1_corner;
            let time_of_collision = - bb_distance /
                (v1.0.xy() - v2.0.xy());
            if time_of_collision.x > 0. && time_of_collision.x < time_of_collision.y.abs() {
                tf1.translation.x += bb_distance.x;
                tf2.translation.x -= bb_distance.x;
            }
            else {
                tf1.translation.y += bb_distance.y / 2.;
                tf2.translation.y -= bb_distance.y / 2.;
                if top_bottom {
                    jump_charge1.0 = true;
                }
                else {
                    jump_charge2.0 = true;
                }
            }
            let v1_new = 2. * (m1.0 * v1.0 + m2.0 * v2.0) / (m1.0 + m2.0) - v1.0;
            let v2_new = 2. * (m1.0 * v1.0 + m2.0 * v2.0) / (m1.0 + m2.0) - v2.0;
            v1.0 = v1_new;
            v2.0 = v2_new;
        }
    }
}

fn spawn_players(mut commands: Commands, asset_server: Res<AssetServer>) {
    let penguin: Handle<Image> = asset_server.load("textures/penguin3.png");
    let seal: Handle<Image> = asset_server.load("textures/seal1.png");
    commands.spawn((
        DespawnOnExit(GameStates::Game),
        PlayerBundle {
        player: Player(1),
        transform: Transform {
            translation: Vec3::new(100.0, 25.0, 0.0),
            scale: Vec2::new(50.0, 50.0).extend(1.0),
            ..default()
        },
        sprite: Sprite {
            image: penguin.clone(),
            custom_size: Some(Vec2::new(1.,1.)),
            ..default()
        },
        ..Default::default()
    }));
    commands.spawn((
        DespawnOnExit(GameStates::Game),
        PlayerBundle {
        player: Player(2),
        transform: Transform {
            translation: Vec3::new(-100.0, 25.0, 0.0),
            scale: Vec2::new(50.0, 50.0).extend(1.0),
            ..default()
        },
        sprite: Sprite {
            image: seal.clone(),
            custom_size: Some(Vec2::new(1.,1.)),
            ..default()
        },
        ..Default::default()
    }));
}

fn spawn_platforms(mut commands: Commands) {
    commands.spawn(
        (
            DespawnOnExit(GameStates::Game),
            Platform,
            Transform {
                translation: Vec3::new(0.0, -150.0, 0.0),
                scale: Vec2::new(600.0, 50.0).extend(1.0),
                ..default()
            },
            Sprite::from_color(Color::srgb(0.7, 0.7, 1.0), Vec2::ONE)
        ));
    commands.spawn(
        (
            DespawnOnExit(GameStates::Game),
            Platform,
            Transform {
                translation: Vec3::new(-600.0, -50.0, 0.0),
                scale: Vec2::new(300.0, 50.0).extend(1.0),
                ..default()
            },
            Sprite::from_color(Color::srgb(0.7, 0.7, 1.0), Vec2::ONE)
        ));
    commands.spawn(
        (
            DespawnOnExit(GameStates::Game),
            Platform,
            Transform {
                translation: Vec3::new(600.0, -50.0, 0.0),
                scale: Vec2::new(300.0, 50.0).extend(1.0),
                ..default()
            },
            Sprite::from_color(Color::srgb(0.7, 0.7, 1.0), Vec2::ONE)
        ));
    commands.spawn(
        (
            DespawnOnExit(GameStates::Game),
            Transform {
             translation: Vec3::new(0.0, -750.0, 1.0),
             scale: Vec2::new(4000.0, 1000.0).extend(1.0),
             ..default()
         },
            Sprite::from_color(Color::srgb(0.0, 0.2, 1.0), Vec2::ONE)
        ));
}

fn spawn_game_soundtrack(mut commands: Commands, asset_server: Res<AssetServer>) {
    let soundtrack = asset_server.load::<AudioSource>("sounds/platform_fighter2.ogg");
    commands.spawn((
        DespawnOnExit(GameStates::Game),
        AudioPlayer(soundtrack.clone()),
        PlaybackSettings::LOOP,
    ));
}

fn spawn_score_display(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/terminal-grotesque.ttf");
    commands.spawn(
        (
            DespawnOnExit(GameStates::Game),
            ScoreDisplay(1),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                left: Val::Px(20.0),
                ..default()
            },
            Text::new("0"),
            TextColor(Color::BLACK),
            TextFont {
                font: font.clone(),
                font_size: 128.,
                ..default()
            },
         ));
    commands.spawn(
        (
            DespawnOnExit(GameStates::Game),
            ScoreDisplay(2),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                right: Val::Px(20.0),
                ..default()
            },
            Text::new("0"),
            TextColor(Color::BLACK),
            TextFont {
                font: font.clone(),
                font_size: 128.,
                ..default()
            },
         ));
}

fn initialize(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
}

fn start_game(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameStates>>,
    keyboard_input: Res<ButtonInput<KeyCode>>
) {
        if keyboard_input.pressed(KeyCode::Enter) {
            next_state.set(GameStates::Game);
        }
}


pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOR));
        app.add_observer(show_score);
        app.add_observer(check_game_over);
        app.add_systems(Startup, (initialize));
        // Menu systems
        app.add_systems(FixedUpdate, (start_game
        ).in_set(MenuSet));
        app.configure_sets(FixedUpdate,(MenuSet.run_if(in_state(GameStates::Menu))));

        // Game systems
        app.add_systems(OnEnter(GameStates::Game), (spawn_players,
                                                    spawn_platforms,
                                                    spawn_game_soundtrack,
                                                    spawn_score_display));
        app.add_systems(FixedUpdate, ((movement_force,
                                      friction_force,
                                      gravitation_force,
                                      jump,
                                      special_move,
                                      apply_velocity,
                                      flip_sprite,
                                      platform_collide,
                                      player_collide).chain(),
                                      respawn,
        ).in_set(GameSet));
        app.configure_sets(FixedUpdate,(
            GameSet.run_if(in_state(GameStates::Game))
        ));

        // GameOver systems
        app.add_systems(OnEnter(GameStates::GameOver), game_over_screen);
        app.add_systems(FixedUpdate, (rematch).in_set(GameOverSet));
        app.configure_sets(FixedUpdate,(
            GameOverSet.run_if(in_state(GameStates::GameOver))
        ));

        app.insert_state(GameStates::Menu);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .run();
}
