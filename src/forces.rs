use std::collections::HashMap;

use bevy::prelude::*;
use itertools::Itertools;

use crate::{groups::Groupable, random::random_direction};

const MAX_SPEED: f32 = 20.0;
const FRICTION_COEFFICIENT: f32 = 0.01;
const COHESION_STRENGTH: f32 = 2.0;
const SEPARATION_STRENGTH: f32 = 50.0;
const SEPARATION_RADIUS: f32 = 2.0;
const ALIGNMENT_STRENGTH: f32 = 0.01;
const WANDER_STRENGTH: f32 = 1.0;

#[derive(Component, Default)]
pub struct Moveable {
    pub velocity: Vec3,
}

#[derive(Component, Default)]
pub struct Forceable;

#[derive(Component, Default)]
pub struct Cohesive {
    force: Vec3,
}

#[derive(Component, Default)]
pub struct Separation {
    force: Vec3,
}

#[derive(Component, Default)]
pub struct Alignment {
    force: Vec3,
}

#[derive(Component, Default)]
pub struct Friction {
    force: Vec3,
}

#[derive(Component, Default)]
pub struct Wander {
    force: Vec3,
}

pub struct ForcesPlugin;

impl Plugin for ForcesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(move_system)
            .add_system(apply_forces_system)
            .add_system(cohesion_force_system)
            .add_system(separation_force_system)
            .add_system(alignment_force_system)
            .add_system(friction_force_system)
            .add_system(wander_force_system);
    }
}

pub fn move_system(time: Res<Time>, mut moveables: Query<(&mut Transform, &Moveable)>) {
    let delta_time = time.delta_seconds();
    moveables.for_each_mut(|(mut transform, moveable)| {
        let delta_position = moveable.velocity * delta_time;
        let next_position = transform.translation + delta_position;
        if delta_position.length() > f32::EPSILON {
            transform.look_at(next_position, Vec3::Z);
        }
        transform.translation = next_position;
    });
}

pub fn apply_forces_system(
    time: Res<Time>,
    mut bodies: Query<
        (
            &mut Moveable,
            Option<&Cohesive>,
            Option<&Separation>,
            Option<&Alignment>,
            Option<&Friction>,
            Option<&Wander>,
        ),
        With<Forceable>,
    >,
) {
    let delta_time = time.delta_seconds();
    bodies.for_each_mut(
        |(mut moveable, cohesive, separation, alignment, friction, wander)| {
            if let Some(c) = cohesive {
                moveable.velocity += delta_time * c.force;
            }

            if let Some(s) = separation {
                moveable.velocity += delta_time * s.force;
            }

            if let Some(a) = alignment {
                moveable.velocity += delta_time * a.force;
            }

            if let Some(f) = friction {
                moveable.velocity += delta_time * f.force;
            }

            if let Some(w) = wander {
                moveable.velocity += delta_time * w.force;
            }

            moveable.velocity = moveable.velocity.clamp_length_max(MAX_SPEED);
        },
    );
}

pub fn cohesion_force_system(mut cohesives: Query<(&Transform, &mut Cohesive, &Groupable)>) {
    let group_com_map: HashMap<usize, Vec3> = cohesives
        .iter()
        .filter_map(|(t, c, g)| match g.id {
            Some(id) => Some((t, c, id)),
            None => None,
        })
        .group_by(|(_, _, id)| *id)
        .into_iter()
        .map(|(id, group)| {
            let (position_summed, count) = group
                .fold((Vec3::ZERO, 0), |(com, count), (t, _, _)| {
                    (com + t.translation, count + 1)
                });

            let group_center_of_mass = (1.0 / count as f32) * position_summed;

            return (id, group_center_of_mass);
        })
        .collect();

    cohesives.for_each_mut(|(t, mut c, g)| {
        if let Some(id) = g.id {
            if let Some(group_center_of_mass) = group_com_map.get(&id) {
                c.force = COHESION_STRENGTH * (*group_center_of_mass - t.translation);
            }
        }
    });
}

pub fn separation_force_system(mut separations: Query<(&Transform, &mut Separation, &Groupable)>) {
    separations.for_each_mut(|(_, mut separation, _)| separation.force = Vec3::ZERO);

    let mut iter = separations.iter_combinations_mut();
    while let Some([(t1, mut s1, g1), (t2, mut s2, g2)]) = iter.fetch_next() {
        if g1.id == None || g2.id == None {
            continue;
        }

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

pub fn wander_force_system(mut wanders: Query<(&mut Wander, &Groupable)>) {
    wanders.for_each_mut(|(mut w, g)| {
        if g.id == None {
            let strength = WANDER_STRENGTH * rand::random::<f32>();
            w.force = strength * random_direction();
        }
    });
}

pub fn alignment_force_system(mut cohesives: Query<(&Moveable, &mut Alignment, &Groupable)>) {
    let group_alignment_force_map: HashMap<usize, Vec3> = cohesives
        .iter()
        .filter_map(|(m, a, g)| match g.id {
            Some(id) => Some((m, a, id)),
            None => None,
        })
        .group_by(|(_, _, id)| *id)
        .into_iter()
        .map(|(id, group)| {
            let (velocity_summed, count) = group
                .fold((Vec3::ZERO, 0), |(velocity, count), (m, _, _)| {
                    (velocity + m.velocity, count + 1)
                });

            let average_velocity = (1.0 / count as f32) * velocity_summed;

            return (id, ALIGNMENT_STRENGTH * average_velocity);
        })
        .collect();

    cohesives.for_each_mut(|(_, mut a, g)| {
        if let Some(id) = g.id {
            if let Some(alignment_force) = group_alignment_force_map.get(&id) {
                a.force = *alignment_force;
            }
        }
    });
}

pub fn friction_force_system(mut moveables: Query<(&Moveable, &mut Friction)>) {
    moveables.for_each_mut(|(fish, mut friction)| {
        friction.force = -1.0 * FRICTION_COEFFICIENT * fish.velocity;
    });
}
