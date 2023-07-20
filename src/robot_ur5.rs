use bevy::prelude::*;
use nalgebra::{matrix, Matrix4};
use std::f64::consts::{FRAC_PI_2, PI};

const ASSET: [&str; 7] = [
    "ur5/ur5.gltf#Scene0",
    "ur5/ur5.gltf#Scene1",
    "ur5/ur5.gltf#Scene2",
    "ur5/ur5.gltf#Scene3",
    "ur5/ur5.gltf#Scene4",
    "ur5/ur5.gltf#Scene5",
    "ur5/ur5.gltf#Scene6",
];

#[derive(Component)]
pub enum RobotComponent {
    Base,
    Arm1,
    Arm2,
    Arm3,
    Arm4,
    Arm5,
    Arm6,
}

pub const JOINTS_POS: [f64; 6] = [90.0, -120.0, 90.0, -60.0, -90.0, 0.0];

#[derive(Component)]
pub struct RobotUr5 {
    pub id: u64,
    joints: [f64; 6], // rad
}

#[derive(Component)]
pub struct RobotWrist(pub u64);

impl RobotUr5 {
    fn get_local_tfs(&self) -> [Transform; 7] {
        let m4s = compute_joint_to_base(self.joints);
        [
            Transform::default(),
            matrix4_to_tf(m4s[0]),
            matrix4_to_tf(m4s[1]),
            matrix4_to_tf(m4s[2]),
            matrix4_to_tf(m4s[3]),
            matrix4_to_tf(m4s[4]),
            matrix4_to_tf(m4s[5]),
        ]
    }

    fn default_joints() -> [f64; 6] {
        let mut out = [0.0; 6];
        for i in 0..6 {
            out[i] = d2r(JOINTS_POS[i]);
        }
        out
    }

    pub fn set_deg(&mut self, j: [f64; 6]) {
        for i in 0..6 {
            self.joints[i] = d2r(j[i]);
        }
    }
}

#[derive(Resource, Clone)]
struct RobotPluginResUr5 {
    base: Handle<Scene>,
    arm1: Handle<Scene>,
    arm2: Handle<Scene>,
    arm3: Handle<Scene>,
    arm4: Handle<Scene>,
    arm5: Handle<Scene>,
    arm6: Handle<Scene>,
}

impl FromWorld for RobotPluginResUr5 {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let base = asset_server.load(ASSET[0]);
        let arm1 = asset_server.load(ASSET[1]);
        let arm2 = asset_server.load(ASSET[2]);
        let arm3 = asset_server.load(ASSET[3]);
        let arm4 = asset_server.load(ASSET[4]);
        let arm5 = asset_server.load(ASSET[5]);
        let arm6 = asset_server.load(ASSET[6]);
        RobotPluginResUr5 {
            base,
            arm1,
            arm2,
            arm3,
            arm4,
            arm5,
            arm6,
        }
    }
}

pub struct RobotPluginUr5;

impl RobotPluginUr5 {
    // (base,arm6)
    pub fn add_robot(
        world: &mut World,
        id: u64,
        tf: Option<Transform>,
        joints: Option<[f64; 6]>, // rad
    ) -> (Entity, Entity) {
        let mut spatial_bundle = SpatialBundle::default();
        if let Some(tf) = tf {
            spatial_bundle.transform = tf;
        } else {
            spatial_bundle
                .transform
                .rotate_x(-std::f32::consts::FRAC_PI_2);
        }
        let joints = joints.unwrap_or(RobotUr5::default_joints());
        let robot = RobotUr5 { id, joints };
        let component_tfs = robot.get_local_tfs();
        let parent = world.spawn((robot, spatial_bundle)).id();

        let res: RobotPluginResUr5 = world.get_resource::<RobotPluginResUr5>().unwrap().clone();
        let child_base = world
            .spawn((
                SceneBundle {
                    scene: res.base.clone(),
                    transform: component_tfs[0],
                    ..default()
                },
                RobotComponent::Base,
            ))
            .id();
        let child_arm1 = world
            .spawn((
                SceneBundle {
                    scene: res.arm1.clone(),
                    transform: component_tfs[1],
                    ..default()
                },
                RobotComponent::Arm1,
            ))
            .id();
        let child_arm2 = world
            .spawn((
                SceneBundle {
                    scene: res.arm2.clone(),
                    transform: component_tfs[2],
                    ..default()
                },
                RobotComponent::Arm2,
            ))
            .id();
        let child_arm3 = world
            .spawn((
                SceneBundle {
                    scene: res.arm3.clone(),
                    transform: component_tfs[3],
                    ..default()
                },
                RobotComponent::Arm3,
            ))
            .id();
        let child_arm4 = world
            .spawn((
                SceneBundle {
                    scene: res.arm4.clone(),
                    transform: component_tfs[4],
                    ..default()
                },
                RobotComponent::Arm4,
            ))
            .id();
        let child_arm5 = world
            .spawn((
                SceneBundle {
                    scene: res.arm5.clone(),
                    transform: component_tfs[5],
                    ..default()
                },
                RobotComponent::Arm5,
            ))
            .id();
        let child_arm6 = world
            .spawn((
                SceneBundle {
                    scene: res.arm6.clone(),
                    transform: component_tfs[6],
                    ..default()
                },
                RobotComponent::Arm6,
                RobotWrist(id),
            ))
            .id();

        let children = [
            child_base, child_arm1, child_arm2, child_arm3, child_arm4, child_arm5, child_arm6,
        ];

        world.entity_mut(parent).push_children(&children);
        (parent, child_arm6)
    }

    fn update_component_pos(
        q_parent: Query<(&RobotUr5, &Children), Changed<RobotUr5>>,
        mut q_child: Query<(&RobotComponent, &mut Transform)>,
    ) {
        for (ur5, children) in q_parent.iter() {
            let tfs = ur5.get_local_tfs();
            for &child in children.iter() {
                if let Ok((rc, mut tf)) = q_child.get_mut(child) {
                    match rc {
                        RobotComponent::Base => {
                            *tf = tfs[0];
                        }
                        RobotComponent::Arm1 => {
                            *tf = tfs[1];
                        }
                        RobotComponent::Arm2 => {
                            *tf = tfs[2];
                        }
                        RobotComponent::Arm3 => {
                            *tf = tfs[3];
                        }
                        RobotComponent::Arm4 => {
                            *tf = tfs[4];
                        }
                        RobotComponent::Arm5 => {
                            *tf = tfs[5];
                        }
                        RobotComponent::Arm6 => {
                            *tf = tfs[6];
                        }
                    }
                }
            }
        }
    }
}

impl Plugin for RobotPluginUr5 {
    fn build(&self, app: &mut App) {
        app.init_resource::<RobotPluginResUr5>()
            .add_systems(Update, RobotPluginUr5::update_component_pos);
    }
}

fn t_(a: f64, alpha: f64, d: f64, theta: f64) -> Matrix4<f64> {
    let i11 = theta.cos();
    let i12 = -theta.sin();
    let i13 = 0.0;
    let i14 = a;
    let i21 = theta.sin() * alpha.cos();
    let i22 = theta.cos() * alpha.cos();
    let i23 = -alpha.sin();
    let i24 = -alpha.sin() * d;
    let i31 = theta.sin() * alpha.sin();
    let i32 = theta.cos() * alpha.sin();
    let i33 = alpha.cos();
    let i34 = alpha.cos() * d;
    let i41 = 0.0;
    let i42 = 0.0;
    let i43 = 0.0;
    let i44 = 1.0;
    let out = matrix![
        i11,i12,i13,i14;
        i21,i22,i23,i24;
        i31,i32,i33,i34;
        i41,i42,i43,i44
    ];
    out
}

struct RobotPar {
    a: f64,
    alpha: f64,
    d: f64,
}

const PARS: [RobotPar; 6] = [
    // 0 - 1
    RobotPar {
        a: 0.0,
        alpha: 0.0,
        d: 89.2 / 1000.0,
    },
    // 1 - 2
    RobotPar {
        a: 0.0,
        alpha: -FRAC_PI_2,
        d: 134.2 / 1000.0,
    },
    // 2 - 3
    RobotPar {
        a: 425.0 / 1000.0,
        alpha: PI,
        d: 118.95 / 1000.0,
    },
    // 3 - 4
    RobotPar {
        a: 392.25 / 1000.0,
        alpha: PI,
        d: 94.75 / 1000.0,
    },
    // 4 - 5
    RobotPar {
        a: 0.0,
        alpha: -FRAC_PI_2,
        d: 94.75 / 1000.0,
    },
    // 5 - 6
    RobotPar {
        a: 0.0,
        alpha: -FRAC_PI_2,
        d: 81.5 / 1000.0,
    },
];

fn compute_joint_to_base(joints: [f64; 6]) -> [Matrix4<f64>; 6] {
    // revised data, align with ur5 robot
    let joints_revise = [
        joints[0] + PI,
        joints[1],
        -joints[2],
        joints[3],
        joints[4] + PI,
        joints[5],
    ];
    let mut ts: [Matrix4<f64>; 6] = [Matrix4::zeros(); 6];
    for i in 0..6 {
        ts[i] = t_(PARS[i].a, PARS[i].alpha, PARS[i].d, joints_revise[i]);
    }
    let mut out = [Matrix4::zeros(); 6];
    let mut buf = ts[0];
    out[0] = buf;
    for i in 1..6 {
        buf = buf * ts[i];
        out[i] = buf;
    }
    out
}

fn matrix4_to_tf(m: Matrix4<f64>) -> Transform {
    let x_axis = Vec4::new(m.m11 as f32, m.m21 as f32, m.m31 as f32, m.m41 as f32);
    let y_axis = Vec4::new(m.m12 as f32, m.m22 as f32, m.m32 as f32, m.m42 as f32);
    let z_axis = Vec4::new(m.m13 as f32, m.m23 as f32, m.m33 as f32, m.m43 as f32);
    let w_axis = Vec4::new(m.m14 as f32, m.m24 as f32, m.m34 as f32, m.m44 as f32);
    let mat = Mat4::from_cols(x_axis, y_axis, z_axis, w_axis);
    Transform::from_matrix(mat)
}

#[inline]
fn d2r(ang: f64) -> f64 {
    ang / 180.0 * PI
}
