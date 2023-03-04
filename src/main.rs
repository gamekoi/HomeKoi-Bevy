use std::f32::consts::PI;

use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_asset_loader::prelude::*;
use camera::*;
use fishes::*;
use forces::ForcesPlugin;
use input::click_to_move_system;

mod camera;
mod fishes;
mod forces;
mod input;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ForcesPlugin)
        .add_system(camera_center_of_mass_track_system)
        .add_system(click_to_move_system)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Running)
                .with_collection::<FishAssets>(),
        )
        .add_state(GameState::AssetLoading)
        .add_system_set(SystemSet::on_enter(GameState::Running).with_system(setup_scene))
        .add_system_set(SystemSet::on_update(GameState::Running).with_system(fish_animator_system))
        .run();
}

const FISH_TO_SPAWN: usize = 100;
const SPAWN_RADIUS: f32 = 50.0;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Running,
}

fn setup_scene(mut commands: Commands, fish_assets: Res<FishAssets>) {
    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::Rgba {
                    red: 0.4,
                    green: 0.75,
                    blue: 0.85,
                    alpha: 1.0,
                }),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        TrackingCenterOfMassCamera,
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 75000.0,
            ..default()
        },
        transform: Transform::from_xyz(10.0, -10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    for _ in 0..FISH_TO_SPAWN {
        let length: f32 = SPAWN_RADIUS * rand::random::<f32>();
        let translation = length * random_direction();
        let direction = random_direction();

        let transform =
            Transform::from_translation(translation).looking_at(translation + direction, Vec3::Z);
        commands.spawn(Fish::new_npc(transform, &fish_assets));
    }

    commands.spawn(Fish::new_player(
        Transform::from_translation(Vec3::ZERO).looking_at(Vec3::Y, Vec3::Z),
        &fish_assets,
    ));
}

fn random_direction() -> Vec3 {
    let angle_ratio: f32 = rand::random();
    let radians = 2.0 * PI * angle_ratio;

    Vec3::new(radians.cos(), radians.sin(), 0.0)
}
