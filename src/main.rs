mod draw_trail;
mod gripper_ctm2f110;
mod robot_ur5;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use flume::{unbounded, Receiver, Sender};
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(target_family = "wasm")]
use serde::{Deserialize, Serialize};

use crate::{
    draw_trail::{DrawTrailPlugin, Trails},
    gripper_ctm2f110::{Finger, GripperCtm2f110, GripperFingertip, GripperPlugin},
    robot_ur5::{RobotPluginUr5, RobotUr5, JOINTS_POS},
};

static mut SENDER: Option<Sender<Cmd>> = None;

#[cfg(target_family = "wasm")]
#[derive(Serialize, Deserialize)]
pub struct EventJointChanged {
    robot: u16,
    joint: u16,
    angle: f32,
}

#[cfg(target_family = "wasm")]
#[derive(Serialize, Deserialize)]
pub struct EventFingerChanged {
    robot: u16,
    finger: u16,
    pos: f32,
}

#[wasm_bindgen]
pub fn set_robot_joint_pos(robot: u16, joint: u16, angle: f32) {
    unsafe {
        if let Some(sender) = &SENDER {
            let cmd = Cmd::RobotJointPos {
                robot,
                joint,
                angle,
            };
            let _ = sender.send(cmd);
        }
    }
}

#[wasm_bindgen]
pub fn set_robot_finger_pos(robot: u16, finger: u16, pos: f32) {
    unsafe {
        if let Some(sender) = &SENDER {
            let cmd = Cmd::RobotFingerPos { robot, finger, pos };
            let _ = sender.send(cmd);
        }
    }
}

const ROBOT_KEY_0: u64 = 0;
const ROBOT_KEY_1: u64 = 1;

fn main() {
    let (sender, receiver) = unbounded::<Cmd>();
    unsafe {
        SENDER = Some(sender);
    }
    let cmd_channel = CmdChannel { receiver };

    App::new()
        .init_resource::<JointsPos>()
        .init_resource::<FingerPos>()
        .insert_resource(cmd_channel)
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                canvas: Some("#demo-bevy_robot".to_string()),
                ..default()
            }),
            ..default()
        }),))
        .add_plugins((
            PanOrbitCameraPlugin,
            EguiPlugin,
            RobotPluginUr5,
            GripperPlugin,
            DrawTrailPlugin,
        ))
        .add_systems(Startup, (setup_camera_light, setup_robot, setup_label))
        .add_systems(
            Update,
            (
                ui,
                update_joints_pos,
                update_finger_pos,
                draw_floor_grids,
                draw_gripper_trails,
                update_label_pos,
                recv_cmd,
            ),
        )
        .run();
}

#[derive(Clone)]
enum Cmd {
    RobotJointPos { robot: u16, joint: u16, angle: f32 },
    RobotFingerPos { robot: u16, finger: u16, pos: f32 },
}

#[derive(Resource)]
struct CmdChannel {
    receiver: Receiver<Cmd>,
}

#[derive(Resource, Clone)]
struct JointsPos([[f64; 6]; 2]); // deg

impl Default for JointsPos {
    fn default() -> Self {
        JointsPos([JOINTS_POS; 2])
    }
}

fn recv_cmd(
    cmd_channel: Res<CmdChannel>,
    mut joints_pos: ResMut<JointsPos>,
    mut finger_pos: ResMut<FingerPos>,
) {
    while let Ok(cmd) = cmd_channel.receiver.try_recv() {
        match cmd {
            Cmd::RobotJointPos {
                robot,
                joint,
                angle,
            } => {
                let robot: usize = if (robot as u64) == ROBOT_KEY_0 { 0 } else { 1 };
                let joint: usize = match joint {
                    1 => 0,
                    2 => 1,
                    3 => 2,
                    4 => 3,
                    5 => 4,
                    _ => 5,
                };
                let angle = if angle > 720.0 {
                    720.0
                } else if angle < -720.0 {
                    -720.0
                } else {
                    angle
                };
                joints_pos.0[robot][joint] = angle as f64;
            }
            Cmd::RobotFingerPos { robot, finger, pos } => {
                let robot: usize = if (robot as u64) == ROBOT_KEY_0 { 0 } else { 1 };
                let finger: usize = match finger {
                    1 => 0,
                    _ => 1,
                };
                let pos = if pos > 100.0 {
                    100.0
                } else if pos < 0.0 {
                    0.0
                } else {
                    pos
                };
                finger_pos.0[robot][finger] = pos;
            }
        }
    }
}

fn update_joints_pos(
    joints: Res<JointsPos>,
    mut query: Query<&mut RobotUr5>,
    mut now_joints: Local<JointsPos>,
    mut last_joints: Local<JointsPos>,
) {
    #[cfg(target_family = "wasm")]
    {
        use std::ops::Deref;
        use web_sys::CustomEvent;
        for robot in 0..=1 {
            for joint in 0..6 {
                if last_joints.0[robot][joint] != joints.0[robot][joint] {
                    let custom_event = CustomEvent::new("").unwrap();
                    let data = EventJointChanged {
                        robot: robot as u16,
                        joint: (joint + 1) as u16,
                        angle: joints.0[robot][joint] as f32,
                    };
                    let js_value = serde_wasm_bindgen::to_value(&data).unwrap();
                    custom_event.init_custom_event_with_can_bubble_and_cancelable_and_detail(
                        "joint_changed",
                        true,
                        true,
                        &js_value,
                    );
                    let window = web_sys::window().unwrap();
                    let _ = window.dispatch_event(custom_event.deref());
                }
            }
        }
    }

    for mut robot in query.iter_mut() {
        let id = robot.id as usize;
        let target = joints.0[id];
        let now = now_joints.0[id];
        let pos = ct_robot_joints(&now, &target);
        now_joints.0[id] = pos;
        robot.set_deg(pos)
    }

    last_joints.0 = joints.0;
}

#[derive(Resource, Clone, Default)]
struct FingerPos([[f32; 2]; 2]); // range [0.0, 100.0]

fn update_finger_pos(
    fingers: Res<FingerPos>,
    mut query: Query<&mut GripperCtm2f110>,
    mut now_fingers: Local<FingerPos>,
    mut last_fingers: Local<FingerPos>,
) {
    #[cfg(target_family = "wasm")]
    {
        use std::ops::Deref;
        use web_sys::CustomEvent;
        for robot in 0..=1 {
            for finger in 0..=1 {
                if last_fingers.0[robot][finger] != fingers.0[robot][finger] {
                    let custom_event = CustomEvent::new("").unwrap();
                    let data = EventFingerChanged {
                        robot: robot as u16,
                        finger: (finger + 1) as u16,
                        pos: fingers.0[robot][finger] as f32,
                    };
                    let js_value = serde_wasm_bindgen::to_value(&data).unwrap();
                    custom_event.init_custom_event_with_can_bubble_and_cancelable_and_detail(
                        "finger_changed",
                        true,
                        true,
                        &js_value,
                    );
                    let window = web_sys::window().unwrap();
                    let _ = window.dispatch_event(custom_event.deref());
                }
            }
        }
    }
    for mut gripper in query.iter_mut() {
        let id = gripper.id as usize;
        let target = fingers.0[id];
        let now = now_fingers.0[id];
        let pos = ct_gripper_finger(&now, &target);
        now_fingers.0[id] = pos;
        gripper.pos1 = pos[0] / 100.0;
        gripper.pos2 = pos[1] / 100.0;
    }

    last_fingers.0 = fingers.0;
}

fn draw_floor_grids(mut gizmos: Gizmos) {
    for i in 0..11 {
        let z = -0.5 + (i as f32) * 0.1;
        gizmos.line(
            Vec3::new(-1.0, 0.0, z),
            Vec3::new(1.0, 0.0, z),
            Color::DARK_GRAY,
        );
    }
    for i in 0..21 {
        let x = -1.0 + (i as f32) * 0.1;
        gizmos.line(
            Vec3::new(x, 0.0, -0.5),
            Vec3::new(x, 0.0, 0.5),
            Color::DARK_GRAY,
        );
    }
}

fn draw_gripper_trails(
    time: Res<Time>,
    mut trails: ResMut<Trails>,
    query_gripper_finger: Query<(&GripperFingertip, &GlobalTransform), Changed<GlobalTransform>>,
) {
    let time = time.elapsed_seconds();
    for (fingertip, global_transform) in query_gripper_finger.iter() {
        if fingertip.finger == Finger::Two {
            continue;
        }
        let id = fingertip.id;
        let point = global_transform.translation();
        trails.add_point(id, time, point, 2.0, Color::GREEN);
    }
}

fn ui(
    mut contexts: EguiContexts,
    mut joints: ResMut<JointsPos>,
    mut finger_pos: ResMut<FingerPos>,
    mut show_window: Local<[bool; 2]>,
) {
    let ctx = contexts.ctx_mut();

    egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if ui.selectable_label(!show_window[0], "Robot0").clicked() {
                    show_window[0] = !(show_window[0]);
                }

                if ui.selectable_label(!show_window[1], "Robot1").clicked() {
                    show_window[1] = !(show_window[1]);
                }
            });
        });

    for i in 0..2 {
        if !show_window[i] {
            egui::Window::new(format!("Robot{}", i)).show(ctx, |ui| {
                if ui.button("reset").clicked() {
                    joints.0[i] = JOINTS_POS;
                    finger_pos.0[i] = [0.0, 0.0];
                }

                egui::Grid::new("robot_axis").num_columns(2).show(ui, |ui| {
                    ui.label("Axis1");
                    ui.add(egui::Slider::new(&mut joints.0[i][0], -360.0..=360.0).suffix("°"));
                    ui.end_row();

                    ui.label("Axis2");
                    ui.add(egui::Slider::new(&mut joints.0[i][1], -360.0..=360.0).suffix("°"));
                    ui.end_row();

                    ui.label("Axis3");
                    ui.add(egui::Slider::new(&mut joints.0[i][2], -360.0..=360.0).suffix("°"));
                    ui.end_row();

                    ui.label("Axis4");
                    ui.add(egui::Slider::new(&mut joints.0[i][3], -360.0..=360.0).suffix("°"));
                    ui.end_row();

                    ui.label("Axis5");
                    ui.add(egui::Slider::new(&mut joints.0[i][4], -360.0..=360.0).suffix("°"));
                    ui.end_row();

                    ui.label("Axis6");
                    ui.add(egui::Slider::new(&mut joints.0[i][5], -360.0..=360.0).suffix("°"));
                    ui.end_row();

                    ui.label("Finger1");
                    ui.add(egui::Slider::new(&mut finger_pos.0[i][0], 0.0..=100.0).suffix("%"));
                    ui.end_row();

                    ui.label("Finger2");
                    ui.add(egui::Slider::new(&mut finger_pos.0[i][1], 0.0..=100.0).suffix("%"));
                    ui.end_row();
                });
            });
        }
    }
}

fn setup_camera_light(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 3.0)),
            ..default()
        },
        PanOrbitCamera {
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Right,
            // modifier_orbit: Some(KeyCode::LShift),
            ..default()
        },
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            illuminance: 20000.0,
            ..default()
        },
        ..default()
    });

    let mut tf = Transform::default();
    tf.rotate_local_x(-std::f32::consts::FRAC_PI_2);
    commands.spawn(DirectionalLightBundle {
        transform: tf,
        directional_light: DirectionalLight {
            shadows_enabled: false,
            illuminance: 20000.0,
            ..default()
        },
        ..default()
    });
}

fn setup_robot(mut commands: Commands) {
    commands.add(|world: &mut World| {
        let mut tf = Transform::from_xyz(-0.5, 0.0, 0.0);
        tf.rotate_x(-std::f32::consts::FRAC_PI_2);
        let (_, wrist) = RobotPluginUr5::add_robot(world, ROBOT_KEY_0, Some(tf), None);
        let (gripper, _, _) =
            GripperPlugin::add_gripper(world, ROBOT_KEY_0, None, Some([0.0, 0.0]));
        world.entity_mut(wrist).push_children(&[gripper]);
    });

    commands.add(|world: &mut World| {
        let mut tf = Transform::from_xyz(0.5, 0.0, 0.0);
        tf.rotate_x(-std::f32::consts::FRAC_PI_2);
        let (_, wrist) = RobotPluginUr5::add_robot(world, ROBOT_KEY_1, Some(tf), None);
        let (gripper, _, _) =
            GripperPlugin::add_gripper(world, ROBOT_KEY_1, None, Some([0.0, 0.0]));
        world.entity_mut(wrist).push_children(&[gripper]);
    });
}

#[derive(Component)]
struct Label(u64);

fn setup_label(mut commands: Commands) {
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Robot0",
                TextStyle {
                    font_size: 24.0,
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },
        Label(ROBOT_KEY_0),
    ));

    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Robot1",
                TextStyle {
                    font_size: 24.0,
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        },
        Label(ROBOT_KEY_1),
    ));
}

fn update_label_pos(
    q_robot: Query<(&RobotUr5, &GlobalTransform)>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_label: Query<(&Label, &mut Style)>,
) {
    if let Ok((camera, camera_gt)) = q_camera.get_single() {
        for (robot, gt) in q_robot.iter() {
            for (label, mut style) in q_label.iter_mut() {
                if robot.id == label.0 {
                    if let Some(pos) = camera.world_to_viewport(camera_gt, gt.translation()) {
                        style.left = Val::Px(pos.x - 40.0);
                        style.top = Val::Px(pos.y + 10.0);
                    }
                }
            }
        }
    }
}

// now 当前值
// target 目标值
// k 变化系数
// d_max 最大变化值
// 返回下一帧的值
fn compute_track(now: f64, target: f64, k: f64, d_max: f64) -> f64 {
    let mut delta = (2.0 * k * (target - now)).abs().sqrt();
    if delta > d_max.abs() {
        delta = d_max.abs();
    }
    if target >= now {
        let out = now + delta;
        if out > target {
            target
        } else {
            out
        }
    } else {
        let out = now - delta;
        if out < target {
            target
        } else {
            out
        }
    }
}

fn ct_robot_joints(now: &[f64; 6], target: &[f64; 6]) -> [f64; 6] {
    let mut out = [0f64; 6];
    for i in 0..6 {
        out[i] = compute_track(now[i], target[i], 0.5, 16.0);
    }
    out
}

fn ct_gripper_finger(now: &[f32; 2], target: &[f32; 2]) -> [f32; 2] {
    let mut out = [0f32; 2];
    for i in 0..2 {
        out[i] = compute_track(now[i] as f64, target[i] as f64, 2.0, 5.0) as f32;
    }
    out
}
