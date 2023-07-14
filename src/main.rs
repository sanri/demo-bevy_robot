mod draw_trail;
mod gripper_ctm2f110;
mod robot_ur5;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::{
    draw_trail::{DrawTrailPlugin, Trails},
    gripper_ctm2f110::{Finger, GripperCtm2f110, GripperFingertip, GripperPlugin},
    robot_ur5::{RobotPluginUr5, RobotUr5, JOINTS_POS},
};

const ROBOT_KEY_0: u64 = 0;
const ROBOT_KEY_1: u64 = 1;

fn main() {
    App::new()
        .init_resource::<JointsPos>()
        .init_resource::<FingerPos>()
        .add_plugins((
            DefaultPlugins,
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
            ),
        )
        .run();
}

#[derive(Resource, Clone)]
struct JointsPos([[f64; 6]; 2]); // deg

impl Default for JointsPos {
    fn default() -> Self {
        JointsPos([JOINTS_POS; 2])
    }
}

fn update_joints_pos(mut query: Query<&mut RobotUr5>, joints: Res<JointsPos>) {
    if joints.is_changed() {
        for mut robot in query.iter_mut() {
            let id = robot.id as usize;
            robot.set_deg(joints.0[id])
        }
    }
}

#[derive(Resource, Clone, Default)]
struct FingerPos([[f32; 2]; 2]); // range [0.0, 100.0]

fn update_finger_pos(mut query: Query<&mut GripperCtm2f110>, fingers: Res<FingerPos>) {
    if fingers.is_changed() {
        for mut gripper in query.iter_mut() {
            let id = gripper.id as usize;
            gripper.pos1 = fingers.0[id][0] / 100.0;
            gripper.pos2 = fingers.0[id][1] / 100.0;
        }
    }
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
