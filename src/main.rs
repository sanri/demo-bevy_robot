mod gripper_ctm2f110;
mod robot_ur5;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::{
    gripper_ctm2f110::{Finger, GripperCtm2f110, GripperFingertip, GripperPlugin},
    robot_ur5::{RobotPluginUr5, RobotUr5, JOINTS_POS},
};

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
        ))
        .add_systems(Startup, (setup_camera_light, setup_robot))
        .add_systems(Update, (ui, update_joints_pos, update_finger_pos))
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
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
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
        let (_, wrist) = RobotPluginUr5::add_robot(world, 0, Some(tf), None);
        let (gripper, _, _) = GripperPlugin::add_gripper(world, 0, None, Some([0.0, 0.0]));
        world.entity_mut(wrist).push_children(&[gripper]);
    });

    commands.add(|world: &mut World| {
        let mut tf = Transform::from_xyz(0.5, 0.0, 0.0);
        tf.rotate_x(-std::f32::consts::FRAC_PI_2);
        let (_, wrist) = RobotPluginUr5::add_robot(world, 1, Some(tf), None);
        let (gripper, _, _) = GripperPlugin::add_gripper(world, 1, None, Some([0.0, 0.0]));
        world.entity_mut(wrist).push_children(&[gripper]);
    });
}
