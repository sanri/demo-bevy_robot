use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

const ASSET: [&str; 4] = [
    "ctm2f110/ctm2f110.gltf#Scene0",
    "ctm2f110/ctm2f110.gltf#Scene1",
    "ctm2f110/ctm2f110.gltf#Scene2",
    "ctm2f110/ctm2f110.gltf#Scene3",
];

const DRIVING_POS: [f32; 2] = [24.0 / 1000.0, 80.75 / 1000.0]; // [ x, z ] mm
const DRIVING_LENGTH: f32 = 55.0 / 1000.0; // mm
const DRIVING_ANGLE: [f32; 2] = [30.0 / 180.0 * PI, 103.5 / 180.0 * PI]; // rad
const FOLLOWER_POS: [f32; 2] = [57.0 / 1000.0, 68.75 / 1000.0]; // [ x, z ] mm
const FINGERTIP_POS: [f32; 2] = [-11.0 / 1000.0, 50.5 / 1000.0]; // mm

#[derive(Component)]
pub enum GripperComponent {
    Main,
    Driving1,
    Driving2,
    Follower1,
    Follower2,
    Finger1,
    Finger2,
    Fingertip1,
    Fingertip2,
}

#[derive(Eq, PartialEq)]
pub enum Finger {
    One,
    Two,
}

#[derive(Component)]
pub struct GripperFingertip {
    pub id: u64,
    pub finger: Finger,
}

#[derive(Component)]
pub struct GripperCtm2f110 {
    pub id: u64,
    pub pos1: f32, // range [ 0.0, 1.0 ]
    pub pos2: f32, // range [ 0.0, 1.0 ]
}

#[derive(Resource, Clone)]
struct GripperPluginRes {
    main: Handle<Scene>,
    driving: Handle<Scene>,
    follower: Handle<Scene>,
    finger: Handle<Scene>,
}

impl FromWorld for GripperPluginRes {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let main = asset_server.load(ASSET[0]);
        let driving = asset_server.load(ASSET[1]);
        let follower = asset_server.load(ASSET[2]);
        let finger = asset_server.load(ASSET[3]);
        GripperPluginRes {
            main,
            driving,
            follower,
            finger,
        }
    }
}

pub struct GripperPlugin;

impl GripperPlugin {
    // (main, fingertip1, fingertip2)
    pub fn add_gripper(
        world: &mut World,
        id: u64,
        tf: Option<Transform>,
        finger_pos: Option<[f32; 2]>,
    ) -> (Entity, Entity, Entity) {
        let mut spatial_bundle = SpatialBundle::default();
        if let Some(tf) = tf {
            spatial_bundle.transform = tf;
        }
        let [pos1, pos2] = finger_pos.unwrap_or([1.0, 1.0]);
        let gripper = GripperCtm2f110 { id, pos1, pos2 };
        let [driving1, follower1, finger1, fingertip1] = compute_finger1(pos1);
        let [driving2, follower2, finger2, fingertip2] = compute_finger2(pos2);
        let parent = world.spawn((gripper, spatial_bundle)).id();

        let res: GripperPluginRes = world.get_resource::<GripperPluginRes>().unwrap().clone();
        let child_main = world
            .spawn((
                SceneBundle {
                    scene: res.main.clone(),
                    ..default()
                },
                GripperComponent::Main,
            ))
            .id();
        let child_driving1 = world
            .spawn((
                SceneBundle {
                    scene: res.driving.clone(),
                    transform: driving1,
                    ..default()
                },
                GripperComponent::Driving1,
            ))
            .id();
        let child_driving2 = world
            .spawn((
                SceneBundle {
                    scene: res.driving.clone(),
                    transform: driving2,
                    ..default()
                },
                GripperComponent::Driving2,
            ))
            .id();
        let child_follower1 = world
            .spawn((
                SceneBundle {
                    scene: res.follower.clone(),
                    transform: follower1,
                    ..default()
                },
                GripperComponent::Follower1,
            ))
            .id();
        let child_follower2 = world
            .spawn((
                SceneBundle {
                    scene: res.follower.clone(),
                    transform: follower2,
                    ..default()
                },
                GripperComponent::Follower2,
            ))
            .id();
        let child_finger1 = world
            .spawn((
                SceneBundle {
                    scene: res.finger.clone(),
                    transform: finger1,
                    ..default()
                },
                GripperComponent::Finger1,
            ))
            .id();
        let child_finger2 = world
            .spawn((
                SceneBundle {
                    scene: res.finger.clone(),
                    transform: finger2,
                    ..default()
                },
                GripperComponent::Finger2,
            ))
            .id();
        let child_fingertip1 = world
            .spawn((
                SpatialBundle {
                    transform: fingertip1,
                    ..default()
                },
                GripperComponent::Fingertip1,
                GripperFingertip {
                    id,
                    finger: Finger::One,
                },
            ))
            .id();
        let child_fingertip2 = world
            .spawn((
                SpatialBundle {
                    transform: fingertip2,
                    ..default()
                },
                GripperComponent::Fingertip2,
                GripperFingertip {
                    id,
                    finger: Finger::Two,
                },
            ))
            .id();
        let children = [
            child_main,
            child_driving1,
            child_driving2,
            child_follower1,
            child_follower2,
            child_finger1,
            child_finger2,
            child_fingertip1,
            child_fingertip2,
        ];
        world.entity_mut(parent).push_children(&children);
        (parent, child_fingertip1, child_fingertip2)
    }

    fn update_component_pos(
        q_parent: Query<(&GripperCtm2f110, &Children), Changed<GripperCtm2f110>>,
        mut q_child: Query<(&GripperComponent, &mut Transform)>,
    ) {
        for (gripper, children) in q_parent.iter() {
            let [driving1, follower1, finger1, fingertip1] = compute_finger1(gripper.pos1);
            let [driving2, follower2, finger2, fingertip2] = compute_finger2(gripper.pos2);
            for &child in children.iter() {
                if let Ok((gc, mut tf)) = q_child.get_mut(child) {
                    match gc {
                        GripperComponent::Main => {}
                        GripperComponent::Driving1 => {
                            *tf = driving1;
                        }
                        GripperComponent::Driving2 => {
                            *tf = driving2;
                        }
                        GripperComponent::Follower1 => {
                            *tf = follower1;
                        }
                        GripperComponent::Follower2 => {
                            *tf = follower2;
                        }
                        GripperComponent::Finger1 => {
                            *tf = finger1;
                        }
                        GripperComponent::Finger2 => {
                            *tf = finger2;
                        }
                        GripperComponent::Fingertip1 => {
                            *tf = fingertip1;
                        }
                        GripperComponent::Fingertip2 => {
                            *tf = fingertip2;
                        }
                    }
                }
            }
        }
    }
}

impl Plugin for GripperPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GripperPluginRes>()
            .add_systems(Update, GripperPlugin::update_component_pos);
    }
}

// In: pos range [0.0, 1.0]
// Out: [driving,follower,finger,fingertip]
fn compute_finger2(pos: f32) -> [Transform; 4] {
    let pos = (1.0 - pos).abs();
    let pos = if pos > 1.0 { 1.0 } else { pos };
    let angle = DRIVING_ANGLE[0] + (DRIVING_ANGLE[1] - DRIVING_ANGLE[0]) * pos;

    let driving_x = DRIVING_POS[0];
    let driving_z = DRIVING_POS[1];
    let mut driving = Transform::from_xyz(driving_x, 0.0, driving_z);
    driving.rotate_local_x(FRAC_PI_2);
    driving.rotate_local_z(angle);

    let mut follower = Transform::from_xyz(FOLLOWER_POS[0], 0.0, FOLLOWER_POS[1]);
    follower.rotate_local_x(FRAC_PI_2);
    follower.rotate_local_z(angle);

    let finger_x = driving_x + DRIVING_LENGTH * angle.cos();
    let finger_z = driving_z + DRIVING_LENGTH * angle.sin();
    let mut finger = Transform::from_xyz(finger_x, 0.0, finger_z);
    finger.rotate_local_x(FRAC_PI_2);
    finger.rotate_local_z(FRAC_PI_2);

    let fingertip_x = finger_x + FINGERTIP_POS[0];
    let fingertip_z = finger_z + FINGERTIP_POS[1];
    let mut fingertip = Transform::from_xyz(fingertip_x, 0.0, fingertip_z);
    fingertip.rotate_local_x(FRAC_PI_2);
    fingertip.rotate_local_z(FRAC_PI_2);

    [driving, follower, finger, fingertip]
}

// In: pos range [0.0, 1.0]
// Out: [driving,follower,finger,fingertip]
fn compute_finger1(pos: f32) -> [Transform; 4] {
    let mut tfs = compute_finger2(pos);
    for tf in &mut tfs {
        tf.translation.x = -tf.translation.x;
        tf.rotate_z(PI);
    }
    tfs
}
