use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier3d::prelude::RigidBody::KinematicPositionBased;

use crate::{
    camera::{Tracked, TrackedZoomOnly},
    forces::{Alignment, Cohesive, Forceable, Friction, Moveable, Separation, Wander},
    groups::{Groupable, GroupableBundle, JoinedPlayerEvent},
    input::ClickToMove,
};

#[derive(AssetCollection, Resource)]
pub struct FishAssets {
    #[asset(path = "models/fish.glb#Scene0")]
    fish_scene: Handle<Scene>,
    #[asset(path = "models/fish.glb#Animation0")]
    fish_animation: Handle<AnimationClip>,
    #[asset(path = "sounds/background.ogg")]
    background_music: Handle<AudioSource>,
    #[asset(path = "sounds/bubbles.ogg")]
    bubbles_sfx: Handle<AudioSource>,
}

#[derive(Component, Default)]
pub struct Fish;

pub fn fish_animator_system(
    fish_assets: Res<FishAssets>,
    fishes: Query<(Entity, &Moveable, With<Fish>)>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    fishes.for_each(|(entity, moveable, _)| {
        for child in children.iter_descendants(entity) {
            if let Ok(mut player) = players.get_mut(child) {
                if player.is_added() {
                    player.play(fish_assets.fish_animation.clone()).repeat();
                }

                let speed = moveable.velocity.length();
                let animation_speed = 1.0 + speed;
                player.set_speed(animation_speed);
            }
        }
    });
}

pub fn fish_joined_player_cue_system(
    ev_joined: EventReader<JoinedPlayerEvent>,
    fish_assets: Res<FishAssets>,
    audio: Res<Audio>,
) {
    if !ev_joined.is_empty() {
        ev_joined.clear();
        audio.play(fish_assets.bubbles_sfx.clone());
    }
}

pub fn fish_track_system(
    mut commands: Commands,
    untracked_fishes: Query<(Entity, &Groupable), (With<Fish>, Without<Tracked>)>,
) {
    untracked_fishes.for_each(|(entity, groupable)| {
        if groupable.is_grouped_with_player() {
            commands.entity(entity).insert(TrackedZoomOnly::default());
        }
    });
}

impl Fish {
    pub fn new_npc(transform: Transform, fish_assets: &Res<FishAssets>) -> impl Bundle {
        (
            SceneBundle {
                scene: fish_assets.fish_scene.clone(),
                transform: transform,
                ..default()
            },
            Fish,
            Moveable::default(),
            Forceable::default(),
            Friction::default(),
            Separation::default(),
            Cohesive::default(),
            Alignment::default(),
            Wander::default(),
            GroupableBundle::new(Groupable::default(), KinematicPositionBased),
        )
    }

    pub fn new_player(transform: Transform, fish_assets: &Res<FishAssets>) -> impl Bundle {
        (
            SceneBundle {
                scene: fish_assets.fish_scene.clone(),
                transform: transform,
                ..default()
            },
            Fish,
            Moveable::default(),
            Separation::default(),
            Cohesive::default(),
            Alignment::default(),
            ClickToMove::default(),
            Tracked::default(),
            GroupableBundle::new(Groupable::player_groupable(), KinematicPositionBased),
        )
    }
}

impl FishAssets {
    pub fn start_background_music(&self, audio: Res<Audio>) {
        audio.play_with_settings(
            self.background_music.clone(),
            PlaybackSettings {
                repeat: true,
                ..default()
            },
        );
    }
}
