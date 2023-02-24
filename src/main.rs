use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Running,
}

#[derive(AssetCollection, Resource)]
struct FishAssets {
    #[asset(path = "models/fish.glb#Scene0")]
    fish_scene: Handle<Scene>,
    #[asset(path = "models/fish.glb#Animation0")]
    fish_animation: Handle<AnimationClip>,
}

#[derive(Component)]
struct Fish;

fn setup_scene(mut commands: Commands, fish_assets: Res<FishAssets>) {
    commands.spawn(Camera3dBundle {
        camera_3d: Camera3d {
            clear_color: ClearColorConfig::Custom(Color::Rgba {
                red: 0.4,
                green: 0.75,
                blue: 0.85,
                alpha: 1.0,
            }),
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

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

    commands.spawn((
        SceneBundle {
            scene: fish_assets.fish_scene.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::Y, Vec3::Z),
            ..default()
        },
        Fish,
    ));
}

fn fish_animator_system(
    fish_assets: Res<FishAssets>,
    fishes: Query<Entity, With<Fish>>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for fish in &fishes {
        for entity in children.iter_descendants(fish) {
            if let Ok(mut player) = players.get_mut(entity) {
                if player.is_added() {
                    player.play(fish_assets.fish_animation.clone()).repeat();
                }
    
                player.set_speed(1.0);
            }
        }
    }
}
