use bevy::prelude::*;

// pub const TIME_STEP: f32 = 1.0 / 60.0;
// pub const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

// pub struct PlayerPlugin;
// impl Plugin for PlayerPlugin {
//     fn build(&self, app: &mut App) {
//         app
//         .add_startup_system(player_movement_system);
//     }
// }

const TIME_STEP: f32 = 1.0 / 60.0;
const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

#[derive(Component)]
pub struct Player {
    /// linear speed in meters per second
    pub movement_speed: f32,
}

pub fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    let mut movement_direction = Vec3::new(0.0, 0.0, 0.0);
    if keyboard_input.pressed(KeyCode::Left) {
        movement_direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        movement_direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        movement_direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        movement_direction.y -= 1.0;
    }

    for (player, mut transform) in query.iter_mut() {
        let movement_distance = player.movement_speed * TIME_STEP;
        let translation_delta = movement_direction * movement_distance;
        transform.translation += translation_delta;

        // bound the player within the invisible level bounds
        let extents = Vec3::from((BOUNDS / 2.0, 0.0));
        transform.translation = transform.translation.min(extents).max(-extents);
    }
}
