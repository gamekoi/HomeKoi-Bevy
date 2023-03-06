use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

const GROUP_DISTANCE: f32 = 10.0;

static AVAILABLE_GROUP_ID: AtomicUsize = AtomicUsize::new(1);

pub struct GroupsPlugin;

impl Plugin for GroupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(group_by_proximity_system)
            .add_system(merge_groups_system)
            .add_event::<MergeGroupsEvent>()
            .add_event::<JoinedPlayerEvent>();
    }
}

#[derive(Bundle)]
pub struct GroupableBundle {
    pub groupable: Groupable,
    pub rigidbody: RigidBody,
    collider: Collider,
    sensor: Sensor,
    active_collision_types: ActiveCollisionTypes,
    active_events: ActiveEvents,
}

impl GroupableBundle {
    pub fn new(groupable: Groupable, rigidbody: RigidBody) -> Self {
        Self {
            groupable: groupable,
            rigidbody: rigidbody,
            collider: Collider::ball(GROUP_DISTANCE / 2.0),
            sensor: Sensor,
            active_collision_types: ActiveCollisionTypes::all(),
            active_events: ActiveEvents::COLLISION_EVENTS,
        }
    }
}

#[derive(Component, Default)]
pub struct Groupable {
    pub id: Option<usize>,
}

impl Groupable {
    pub fn player_groupable() -> Self {
        Groupable { id: Some(0) }
    }

    pub fn is_grouped_with_player(self: &Self) -> bool {
        if let Some(0) = self.id {
            return true;
        }

        false
    }
}

struct MergeGroupsEvent(usize, usize);

pub struct JoinedPlayerEvent;

fn group_by_proximity_system(
    mut ev_collision: EventReader<CollisionEvent>,
    mut groupables: Query<&mut Groupable>,
    mut ev_merge: EventWriter<MergeGroupsEvent>,
    mut ev_join: EventWriter<JoinedPlayerEvent>,
) {
    for collion in ev_collision.iter() {
        if let CollisionEvent::Started(e1, e2, _) = collion {
            if let Ok([mut g1, mut g2]) = groupables.get_many_mut([*e1, *e2]) {
                match (g1.id, g2.id) {
                    (None, None) => {
                        let group_id = AVAILABLE_GROUP_ID.fetch_add(1, Ordering::Relaxed);
                        g1.id = Some(group_id);
                        g2.id = Some(group_id);
                    }
                    (Some(id), None) => {
                        if id == 0 {
                            ev_join.send(JoinedPlayerEvent);
                        }
                        g2.id = Some(id)
                    }
                    (None, Some(id)) => {
                        if id == 0 {
                            ev_join.send(JoinedPlayerEvent);
                        }
                        g1.id = Some(id)
                    }
                    (Some(id1), Some(id2)) => {
                        if id1 != id2 {
                            if id1 == 0 || id2 == 0 {
                                ev_join.send(JoinedPlayerEvent);
                            }
                            ev_merge.send(MergeGroupsEvent(id1, id2));
                        }
                    }
                }
            }
        }
    }
}

fn merge_groups_system(
    mut groupables: Query<&mut Groupable>,
    mut ev_merge: EventReader<MergeGroupsEvent>,
) {
    let replacement_map: HashMap<usize, usize> = ev_merge
        .iter()
        .map(|MergeGroupsEvent(id1, id2)| (usize::max(*id1, *id2), usize::min(*id1, *id2)))
        .collect();

    groupables.for_each_mut(|mut groupable| {
        if let Some(id) = groupable.id {
            if let Some(new_id) = replacement_map.get(&id) {
                groupable.id = Some(*new_id);
            }
        }
    })
}
