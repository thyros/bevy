use bevy::{math::Vec3, math::Vec3Swizzles, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod enemy;
mod player;

pub const CLEAR: Color = Color::rgb(0.1, 0.5, 1.0);
pub const RESOLUTION: f32 = 16.0 / 9.0;
const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Component)]
struct AnimationTimer {
    timer: Timer,
    min_sprite: usize,
    max_sprite: usize,
}

/// rotate to face player ship behavior
#[derive(Component)]
struct RotateToPlayer {
    /// rotation speed in radians per second
    rotation_speed: f32,
}

#[derive(Resource, Deref)]
struct CharacterSheet(Handle<TextureAtlas>);

fn main() {
    let height: f32 = 900.0;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: height * RESOLUTION,
                height: height,
                title: "My-Bevy".to_owned(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                resizable: false,
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_plugin(WorldInspectorPlugin)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup)
        .add_startup_system(spawn_player)
        .add_startup_system(spawn_enemy)
        .add_system(animate_sprite)
        .add_system(rotate_to_player_system)
        .add_system(player::player_movement_system)
        .run();
}

fn rotate_to_player_system(
    mut query: Query<(&RotateToPlayer, &mut Transform), Without<player::Player>>,
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
        // if sprite.index < timer.min_sprite {
        //     sprite.index = timer.min_sprite;
        // }

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
        player::Player {
            movement_speed: 500.0,
        },
    ));
}

fn spawn_enemy(mut commands: Commands, texture_atlas_handle: Res<CharacterSheet>) {
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.0.clone(),
            transform: Transform {
                translation: Vec3::new(50.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        },
        AnimationTimer {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            min_sprite: 3,
            max_sprite: 5,
        },
        enemy::Enemy {},
        RotateToPlayer {
            rotation_speed: f32::to_radians(90.0), // degrees per second
        },
    ));
}
