use bevy::prelude::*;
use std::collections::{BTreeMap, VecDeque};

pub struct DrawTrailPlugin;

impl Plugin for DrawTrailPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Trails::default())
            .add_systems(Update, Trails::draw_trails);
    }
}

struct Trail {
    duration: f32,
    color: Color,
    data: VecDeque<(f32, Vec3)>, // ( time, point )
}

#[derive(Resource)]
pub struct Trails {
    map: BTreeMap<u64, Trail>,
}

impl Trails {
    pub fn add_point(&mut self, id: u64, time: f32, point: Vec3, duration: f32, color: Color) {
        let trail = self.map.entry(id).or_insert(Trail {
            duration,
            color,
            data: VecDeque::new(),
        });

        trail.duration = duration.abs();
        trail.color = color;
        trail.data.push_back((time.abs(), point));
    }

    fn draw_trails(mut gizmos: Gizmos, mut trails: ResMut<Trails>, time: Res<Time>) {
        let time = time.elapsed_seconds();
        for trail in trails.map.values_mut() {
            // remove timeout point
            'la: while let Some((t, _)) = trail.data.front() {
                if (time - *t) > trail.duration {
                    trail.data.pop_front();
                } else {
                    break 'la;
                }
            }

            // draw line
            let mut point_old: Option<Vec3> = None;
            for (_, point_new) in trail.data.iter() {
                if let Some(point_old) = point_old {
                    gizmos.line(point_old, point_new.clone(), trail.color);
                }
                point_old = Some(point_new.clone());
            }
        }
    }
}

impl Default for Trails {
    fn default() -> Self {
        Trails {
            map: BTreeMap::new(),
        }
    }
}
