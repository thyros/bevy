use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{math::Vec3, math::Vec3Swizzles, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_debug_lines::*;
use rand::Rng;

mod enemy;
mod player;

pub const CLEAR: Color = Color::rgb(0.1, 0.5, 1.0);
pub const RESOLUTION: f32 = 16.0 / 9.0;
const TIME_STEP: f32 = 1.0 / 60.0;
const ENEMIES_COUNT: i32 = 100;
pub const HEIGHT: f32 = 900.0;
pub const WIDTH: f32 = HEIGHT * RESOLUTION;

#[derive(Component)]
struct AnimationTimer {
    timer: Timer,
    min_sprite: usize,
    max_sprite: usize,
}

#[derive(Component)]
struct GunTimer {
    timer: Timer,
}

#[derive(Resource, Deref)]
struct CharacterSheet(Handle<TextureAtlas>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: WIDTH,
                height: HEIGHT,
                title: "My-Bevy".to_owned(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                resizable: false,
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(DebugLinesPlugin::default())
        .add_startup_system_to_stage(StartupStage::PreStartup, setup)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_enemy)
        .add_system(animate_sprite)
        //        .add_system(rotate_to_player_system)
        .add_system(player::player_movement_system)
        .add_system(shoot_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn find_closest(
    query: &Query<(Entity, &enemy::Enemy, &Transform), Without<GunTimer>>,
    pos: Vec3,
    max_dist: f32,
) -> Option<(Entity, Transform)> {
    let mut min: f32 = 5000.0;
    let mut closest: Option<(Entity, Transform)> = None;

    for (entity, enemy, transform) in query.iter() {
        let distance = (pos - transform.translation).length();
        if distance < min && distance < max_dist {
            min = distance;
            closest = Some((entity, *transform));
        }
    }
    return closest;
}

fn shoot_system(
    mut commands: Commands,
    mut lines: ResMut<DebugLines>,
    time: Res<Time>,
    query: Query<(Entity, &enemy::Enemy, &Transform), Without<GunTimer>>,
    mut shoot_query: Query<(&mut GunTimer, &Transform)>,
) {
    for (mut guntimer, guntransform) in &mut shoot_query {
        guntimer.timer.tick(time.delta());
        if guntimer.timer.just_finished() {
            println!("Ticked");
            let closest = find_closest(&query, guntransform.translation, 250.0);
            if let Some((e, t)) = closest {
                lines.line(guntransform.translation, t.translation, 0.1);
                commands.entity(e).despawn();
            }
        }
    }
}

fn rotate_to_player_system(
    mut query: Query<(&enemy::Enemy, &mut Transform), Without<player::Player>>,
    player_query: Query<&Transform, With<player::Player>>,
) {
    let player_transform = player_query.single();
    // get the player translation in 2D
    let player_translation = player_transform.translation.xy();

    for (config, mut enemy_transform) in &mut query {
        // get the enemy ship forward vector in 2D (already unit length)
        let enemy_forward = (enemy_transform.rotation * Vec3::Y).xy();

        // get the vector from the enemy ship to the player ship in 2D and normalize it.
        let to_player = (player_translation - enemy_transform.translation.xy()).normalize();

        // get the dot product between the enemy forward vector and the direction to the player.
        let forward_dot_player = enemy_forward.dot(to_player);

        // if the dot product is approximately 1.0 then the enemy is already facing the player and
        // we can early out.
        if (forward_dot_player - 1.0).abs() < f32::EPSILON {
            continue;
        }

        // get the right vector of the enemy ship in 2D (already unit length)
        let enemy_right = (enemy_transform.rotation * Vec3::X).xy();

        // get the dot product of the enemy right vector and the direction to the player ship.
        // if the dot product is negative them we need to rotate counter clockwise, if it is
        // positive we need to rotate clockwise. Note that `copysign` will still return 1.0 if the
        // dot product is 0.0 (because the player is directly behind the enemy, so perpendicular
        // with the right vector).
        let right_dot_player = enemy_right.dot(to_player);

        // determine the sign of rotation from the right dot player. We need to negate the sign
        // here as the 2D bevy co-ordinate system rotates around +Z, which is pointing out of the
        // screen. Due to the right hand rule, positive rotation around +Z is counter clockwise and
        // negative is clockwise.
        let rotation_sign = -f32::copysign(1.0, right_dot_player);

        // limit rotation so we don't overshoot the target. We need to convert our dot product to
        // an angle here so we can get an angle of rotation to clamp against.
        let max_angle = forward_dot_player.clamp(-1.0, 1.0).acos(); // clamp acos for safety

        // calculate angle of rotation with limit
        let rotation_angle = rotation_sign * (config.rotation_speed * TIME_STEP).min(max_angle);

        // rotate the enemy to face the player
        enemy_transform.rotate_z(rotation_angle);

        let delta = config.movement_speed * TIME_STEP;
        let step: Vec3 = Vec3::new(enemy_forward.x * delta, enemy_forward.y * delta, 0.0);
        enemy_transform.translation += step;
    }
}

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            let mut new_index = sprite.index + 1;
            if new_index > timer.max_sprite {
                new_index = timer.min_sprite;
            }
            sprite.index = new_index;
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("character.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 32.0), 12, 8, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(CharacterSheet(texture_atlas_handle));
    commands.spawn(Camera2dBundle::default());
}

fn spawn_player(mut commands: Commands, texture_atlas_handle: Res<CharacterSheet>) {
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.0.clone(),
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..default()
        },
        AnimationTimer {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            min_sprite: 0,
            max_sprite: 2,
        },
        GunTimer {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        },
        player::Player {
            movement_speed: 500.0,
        },
    ));
}

fn spawn_enemy(mut commands: Commands, texture_atlas_handle: Res<CharacterSheet>) {
    let maxx: i32 = (WIDTH / 2.0) as i32;
    let minx: i32 = -maxx;
    let maxy: i32 = (HEIGHT / 2.0) as i32;
    let miny: i32 = -maxy;

    for _i in 0..ENEMIES_COUNT {
        let position: Vec3 = Vec3::new(
            rand::thread_rng().gen_range(minx..maxx) as f32,
            rand::thread_rng().gen_range(miny..maxy) as f32,
            0.0,
        );

        commands.spawn((
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.0.clone(),
                transform: Transform {
                    translation: position,
                    ..default()
                },
                ..default()
            },
            AnimationTimer {
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                min_sprite: 3,
                max_sprite: 5,
            },
            enemy::Enemy {
                rotation_speed: f32::to_radians(90.0), // degrees per second
                movement_speed: 100.0,
            },
        ));
    }
}
