use bevy::prelude::*;
use std::f32::consts::PI;

pub fn random_direction() -> Vec3 {
    let angle_ratio: f32 = rand::random();
    let radians = 2.0 * PI * angle_ratio;

    Vec3::new(radians.cos(), radians.sin(), 0.0)
}
