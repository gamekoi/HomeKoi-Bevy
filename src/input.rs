use crate::forces::Moveable;
use bevy::prelude::*;

const MAX_SPEED: f32 = 30.0;

#[derive(Component, Default)]
pub struct ClickToMove;

pub fn click_to_move_system(
    windows: Res<Windows>,
    buttons: Res<Input<MouseButton>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut moveables_to_push: Query<(&mut Moveable, &Transform, With<ClickToMove>)>,
) {
    if buttons.pressed(MouseButton::Left) {
        if let Some(target) = try_get_cursor_position_on_z_plane(windows, cameras) {
            moveables_to_push.for_each_mut(|(mut moveable, transform, _)| {
                let mut delta = target - transform.translation;
                delta.z = 0.0;
                moveable.velocity = delta.clamp_length_max(MAX_SPEED);
            });

            return;
        }
    }

    moveables_to_push.for_each_mut(|(mut moveable, _, _)| {
        moveable.velocity = Vec3::ZERO;
    });
}

pub fn try_get_cursor_position_on_z_plane(
    windows: Res<Windows>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec3> {
    if let Ok((camera, camera_transform)) = cameras.get_single() {
        if let Some(window) = windows.get_primary() {
            if let Some(viewport_position) = window.cursor_position() {
                if let Some(ray) = camera.viewport_to_world(camera_transform, viewport_position) {
                    let dt = -ray.origin.z / ray.direction.z;
                    let intersection_on_z_plane = ray.origin + dt * ray.direction;

                    return Some(intersection_on_z_plane);
                }
            }
        }
    }
    None
}
