use bevy::prelude::*;

#[derive(Component)]
pub struct Enemy {
    pub rotation_speed: f32,
    pub movement_speed: f32,
}
