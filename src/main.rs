use bevy::{core_pipeline::clear_color::ClearColorConfig, gltf::Gltf, prelude::*};
use bevy_asset_loader::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::SetupScene)
                .with_collection::<FishAssets>(),
        )
        .add_state(GameState::AssetLoading)
        .add_system_set(SystemSet::on_enter(GameState::SetupScene).with_system(setup_scene))
        .add_system_set(SystemSet::on_update(GameState::SetupScene).with_system(start_animation))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    SetupScene,
    Running,
}

#[derive(AssetCollection, Resource)]
struct FishAssets {
    #[asset(path = "models/fish.glb")]
    fish_gltf: Handle<Gltf>,
}

fn setup_scene(
    mut commands: Commands,
    fish_assets: Res<FishAssets>,
    gltf_assets: Res<Assets<Gltf>>,
) {
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

    if let Some(fish_gltf) = gltf_assets.get(&fish_assets.fish_gltf) {
        commands.spawn(SceneBundle {
            scene: fish_gltf.default_scene.clone().unwrap(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::Y, Vec3::Z),
            ..default()
        });
    }
}

fn start_animation(
    fish_assets: Res<FishAssets>,
    gltf_assets: Res<Assets<Gltf>>,
    mut player: Query<&mut AnimationPlayer>,
    mut animations_started: Local<bool>,
    mut state: ResMut<State<GameState>>,
) {
    if *animations_started {
        if let Err(err) = state.set(GameState::Running) {
            println!("Failed to transition to running {err:?}");
        }
        return;
    }

    if let Some(gltf) = gltf_assets.get(&fish_assets.fish_gltf) {
        let animation = gltf.animations[0].clone();

        let res = player.get_single_mut();
        if let Ok(mut player) = res {
            player.play(animation).repeat();
            *animations_started = true;
        } else if let Err(err) = res {
            println!("Error getting player {err:?}");
        }
    }
}
