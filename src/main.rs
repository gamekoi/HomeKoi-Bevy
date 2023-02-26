use std::f32::consts::PI;

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
        .add_system_set(SystemSet::on_update(GameState::Running).with_system(fish_move_system))
        .add_system_set(
            SystemSet::on_update(GameState::Running).with_system(fish_apply_forces_system),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Running).with_system(fish_cohesion_force_system),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Running).with_system(fish_separation_force_system),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Running).with_system(fish_alignment_force_system),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Running).with_system(fish_friction_force_system),
        )
        .run();
}

const FISH_TO_SPAWN: usize = 100;
const SPAWN_RADIUS: f32 = 50.0;
const MAX_SPEED: f32 = 20.0;
const FRICTION_COEFFICIENT: f32 = 0.01;
const COHESION_STRENGTH: f32 = 0.75;
const SEPARATION_STRENGTH: f32 = 50.0;
const SEPARATION_RADIUS: f32 = 2.0;
const ALIGNMENT_STRENGTH: f32 = 0.1;

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

#[derive(Component, Default)]
struct Fish {
    velocity: Vec3,
}

#[derive(Component, Default)]
struct Cohesive {
    force: Vec3,
}

#[derive(Component, Default)]
struct Separation {
    force: Vec3,
}

#[derive(Component, Default)]
struct Alignment {
    force: Vec3,
}

#[derive(Component, Default)]
struct Friction {
    force: Vec3,
}

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
        transform: Transform::from_xyz(0.0, 0.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    for _ in 0..FISH_TO_SPAWN {
        let length: f32 = SPAWN_RADIUS * rand::random::<f32>();
        let translation = length * random_direction();
        let direction = random_direction();

        commands.spawn((
            SceneBundle {
                scene: fish_assets.fish_scene.clone(),
                transform: Transform::from_translation(translation)
                    .looking_at(translation + direction, Vec3::Z),
                ..default()
            },
            Fish::default(),
            Friction::default(),
            Separation::default(),
            Cohesive::default(),
            Alignment::default(),
        ));
    }
}

fn random_direction() -> Vec3 {
    let angle_ratio: f32 = rand::random();
    let radians = 2.0 * PI * angle_ratio;

    Vec3::new(radians.cos(), radians.sin(), 0.0)
}

fn fish_animator_system(
    fish_assets: Res<FishAssets>,
    fishes: Query<(Entity, &Fish)>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    fishes.for_each(|(entity, fish)| {
        for child in children.iter_descendants(entity) {
            if let Ok(mut player) = players.get_mut(child) {
                if player.is_added() {
                    player.play(fish_assets.fish_animation.clone()).repeat();
                }

                let speed = fish.velocity.length();
                let animation_speed = 1.0 + speed;
                player.set_speed(animation_speed);
            }
        }
    });
}

fn fish_move_system(time: Res<Time>, mut fishes: Query<(&mut Transform, &Fish)>) {
    let delta_time = time.delta_seconds();
    fishes.for_each_mut(|(mut transform, fish)| {
        let delta_position = fish.velocity * delta_time;
        let next_position = transform.translation + delta_position;
        if delta_position.length() > f32::EPSILON {
            transform.look_at(next_position, Vec3::Z);
        }
        transform.translation = next_position;
    });
}

fn fish_apply_forces_system(
    time: Res<Time>,
    mut fishes: Query<(
        &mut Fish,
        Option<&Cohesive>,
        Option<&Separation>,
        Option<&Alignment>,
        Option<&Friction>,
    )>,
) {
    let delta_time = time.delta_seconds();
    fishes.for_each_mut(|(mut fish, cohesive, separation, alignment, friction)| {
        if let Some(c) = cohesive {
            fish.velocity += delta_time * c.force;
        }

        if let Some(s) = separation {
            fish.velocity += delta_time * s.force;
        }

        if let Some(a) = alignment {
            fish.velocity += delta_time * a.force;
        }

        if let Some(f) = friction {
            fish.velocity += delta_time * f.force;
        }

        fish.velocity = fish.velocity.clamp_length(0.0, MAX_SPEED);
    });
}

fn fish_cohesion_force_system(mut cohesives: Query<(&Transform, &mut Cohesive)>) {
    let positions_summed: Vec3 = cohesives
        .iter()
        .map(|(transform, _)| transform.translation)
        .sum();

    let count = cohesives.iter().count();
    let center_of_mass = (1.0 / count as f32) * positions_summed;
    cohesives
        .for_each_mut(|(t, mut c)| c.force = COHESION_STRENGTH * (center_of_mass - t.translation));
}

fn fish_separation_force_system(mut separations: Query<(&Transform, &mut Separation)>) {
    separations.for_each_mut(|(_, mut separation)| separation.force = Vec3::ZERO);

    let mut iter = separations.iter_combinations_mut();
    while let Some([(t1, mut s1), (t2, mut s2)]) = iter.fetch_next() {
        let delta = t1.translation - t2.translation;
        let distance = delta.length().abs();

        if distance > f32::EPSILON {
            let r = distance / SEPARATION_RADIUS;
            let r3 = r * r * r;
            let separation_impulse = (SEPARATION_STRENGTH / r3) * delta;
            s1.force += separation_impulse;
            s2.force -= separation_impulse;
        }
    }
}

fn fish_alignment_force_system(mut cohesives: Query<(&Fish, &mut Alignment)>) {
    let velocity_summed: Vec3 = cohesives.iter().map(|(fish, _)| fish.velocity).sum();

    let count = cohesives.iter().count();
    let average_velocity = (1.0 / count as f32) * velocity_summed;
    let alignment_force = ALIGNMENT_STRENGTH * average_velocity;
    cohesives.for_each_mut(|(_, mut c)| c.force = alignment_force);
}

fn fish_friction_force_system(mut fishes: Query<(&Fish, &mut Friction)>) {
    fishes.for_each_mut(|(fish, mut friction)| {
        friction.force = -1.0 * FRICTION_COEFFICIENT * fish.velocity;
    });
}
