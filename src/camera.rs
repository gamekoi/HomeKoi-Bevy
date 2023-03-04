use bevy::prelude::*;

const CAMERA_TRACKING_DISTANCE_SCALE: f32 = 2.44948974278;
const CAMERA_MIN_DISTANCE: f32 = 50.0;
const CAMERA_TRACKING_ZOOM: f32 = 1.0;
const CAMERA_TRACKING_DELAY_LERP: f32 = 0.5;

#[derive(Component, Default)]
pub struct TrackingCenterOfMassCamera;

#[derive(Component, Default)]
pub struct Tracked;

pub fn camera_center_of_mass_track_system(
    mut camera: Query<(&Camera, &mut Transform, With<TrackingCenterOfMassCamera>)>,
    trackables: Query<(&Transform, With<Tracked>, Without<Camera>)>,
) {
    let positions_summed: Vec3 = trackables
        .iter()
        .map(|(transform, _, _)| transform.translation)
        .sum();

    let count = trackables.iter().count();
    let center_of_mass = (1.0 / count as f32) * positions_summed;

    let furthest_distance_squared = trackables
        .iter()
        .map(|(transform, _, _)| transform.translation.distance_squared(center_of_mass))
        .fold(0.0_f32, |d1, d2| d1.max(d2));

    let furthest_distance = furthest_distance_squared.sqrt();

    if let Ok((_, mut transform, _)) = camera.get_single_mut() {
        let target = Vec3::new(
            center_of_mass.x,
            center_of_mass.y,
            f32::max(
                CAMERA_TRACKING_ZOOM * CAMERA_TRACKING_DISTANCE_SCALE * furthest_distance,
                CAMERA_MIN_DISTANCE,
            ),
        );
        transform.translation = transform
            .translation
            .lerp(target, CAMERA_TRACKING_DELAY_LERP);
    }
}
