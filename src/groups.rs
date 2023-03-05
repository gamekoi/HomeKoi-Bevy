use bevy::prelude::*;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

const GROUP_DISTANCE: f32 = 10.0;
const GROUP_DISTANCE_SQUARED: f32 = GROUP_DISTANCE * GROUP_DISTANCE;

static AVAILABLE_GROUP_ID: AtomicUsize = AtomicUsize::new(0);

pub struct GroupsPlugin;

impl Plugin for GroupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(group_by_proximity_system)
            .add_system(merge_groups_system)
            .add_event::<MergeGroupsEvent>();
    }
}

#[derive(Component, Default)]
pub struct Groupable {
    pub id: Option<usize>,
}

struct MergeGroupsEvent(usize, usize);

fn group_by_proximity_system(
    mut groupables: Query<(&mut Groupable, &Transform)>,
    mut ev_merge: EventWriter<MergeGroupsEvent>,
) {
    let mut iter = groupables.iter_combinations_mut();
    while let Some([(mut g1, t1), (mut g2, t2)]) = iter.fetch_next() {
        if t1.translation.distance_squared(t2.translation) < GROUP_DISTANCE_SQUARED {
            match (g1.id, g2.id) {
                (None, None) => {
                    let group_id = AVAILABLE_GROUP_ID.fetch_add(1, Ordering::Relaxed);
                    g1.id = Some(group_id);
                    g2.id = Some(group_id);
                }
                (Some(id), None) => g2.id = Some(id),
                (None, Some(id)) => g1.id = Some(id),
                (Some(id1), Some(id2)) => {
                    if id1 != id2 {
                        ev_merge.send(MergeGroupsEvent(id1, id2));
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