//! Demonstrates Continuous Collision Detection (CCD) to prevent tunneling
//! when fast-moving balls collide with fast-spinning thin objects.
//!
//! Two modes can be toggled between:
//!
//! 1. Linear: Only considers translational motion.
//! 2. Non-linear: Considers both translational and rotational motion. More expensive.
//!
//! This example also demonstrates one CCD problem known as "time loss".
//! To hit the balls correctly, the spinners need to stop at the first time of impact,
//! causing them to lose the rest of their rotation for that frame. This can make
//! the spinners look like they are stuttering, especially at high speeds and with
//! many objects to collide with. A fix for this would require a substepping CCD solver,
//! which is not currently implemented in Avian due to its high overhead and complexity.

use avian2d::{math::*, prelude::*};
use bevy::prelude::*;
use examples_common_2d::ExampleCommonPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ExampleCommonPlugin,
            PhysicsPlugins::default(),
        ))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(Gravity(Vector::NEG_Y * 9.81 * 100.0))
        .init_resource::<ActiveSweepMode>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_config)
        .add_systems(FixedUpdate, spawn_balls)
        .run();
}

#[derive(Component)]
struct SweepModeText;

/// The current CCD sweep mode configuration.
#[derive(Resource, Clone, Copy, Default, PartialEq)]
enum ActiveSweepMode {
    #[default]
    NonLinear,
    Linear,
    Off,
}

impl ActiveSweepMode {
    fn ccd_settings(self) -> CcdSettings {
        match self {
            ActiveSweepMode::NonLinear => CcdSettings::default().with_include_dynamic(true),
            ActiveSweepMode::Linear => CcdSettings::default()
                .with_mode(SweepMode::Linear)
                .with_include_dynamic(true),
            ActiveSweepMode::Off => CcdSettings::default().with_enabled(false),
        }
    }

    fn label(self) -> &'static str {
        match self {
            ActiveSweepMode::NonLinear => "Non-linear (considers both translation and rotation)",
            ActiveSweepMode::Linear => "Linear (considers only translation)",
            ActiveSweepMode::Off => "Off (discrete collision detection only)",
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let sweep_mode = ActiveSweepMode::default();
    let ccd_settings = sweep_mode.ccd_settings();

    // Add two dynamic bodies spinning at high speeds. Their translation is locked and they have a
    // high dominance, so they spin in place without being pushed around.
    commands.spawn((
        Name::new("Spinner 1"),
        Transform::from_xyz(-200.1, -200.0, 0.0),
        RigidBody::Dynamic,
        LockedAxes::TRANSLATION_LOCKED,
        AngularVelocity(25.0),
        Dominance(1),
        Collider::rectangle(1.0, 400.0),
        ccd_settings,
        Sprite {
            color: Color::srgb(0.7, 0.7, 0.8),
            custom_size: Some(Vec2::new(1.0, 400.0)),
            ..default()
        },
    ));
    commands.spawn((
        Name::new("Spinner 2"),
        Transform::from_xyz(200.1, -200.0, 0.0),
        RigidBody::Dynamic,
        LockedAxes::TRANSLATION_LOCKED,
        AngularVelocity(-25.0),
        Dominance(1),
        Collider::rectangle(1.0, 400.0),
        ccd_settings,
        Sprite {
            color: Color::srgb(0.7, 0.7, 0.8),
            custom_size: Some(Vec2::new(1.0, 400.0)),
            ..default()
        },
    ));

    // Setup help text.
    let font = TextFont {
        font_size: 20.0,
        ..default()
    };
    commands.spawn((
        Text::new("Press Space to toggle CCD mode"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        font.clone(),
    ));
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            ..default()
        },
        children![
            (TextSpan::new("CCD: "), font.clone()),
            (TextSpan::new(sweep_mode.label()), font, SweepModeText),
        ],
    ));
}

/// Spawns balls moving at the spinning objects at high speeds.
fn spawn_balls(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time<Physics>>,
    sweep_mode: Res<ActiveSweepMode>,
) {
    let circle = Circle::new(2.0);

    // Compute the shooting direction.
    let (sin, cos) = (0.5 * time.elapsed_secs_f64().adjust_precision().sin() - PI / 2.0).sin_cos();
    let direction = Vector::new(cos, sin).rotate(Vector::X);

    commands.spawn((
        Name::new("Ball"),
        Mesh2d(meshes.add(circle)),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.7, 0.9))),
        Transform::from_xyz(0.0, 350.0, 0.0),
        RigidBody::Dynamic,
        LinearVelocity(2000.0 * direction),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Collider::from(circle),
        sweep_mode.ccd_settings(),
    ));
}

fn update_config(
    sweep_mode_text: Single<&mut TextSpan, With<SweepModeText>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sweep_mode: ResMut<ActiveSweepMode>,
    balls: Query<Entity, With<Collider>>,
    mut commands: Commands,
) {
    // Cycle the CCD configuration for the colliders.
    if keys.just_pressed(KeyCode::Space) {
        let mut text = sweep_mode_text;
        *sweep_mode = match *sweep_mode {
            ActiveSweepMode::NonLinear => ActiveSweepMode::Linear,
            ActiveSweepMode::Linear => ActiveSweepMode::Off,
            ActiveSweepMode::Off => ActiveSweepMode::NonLinear,
        };
        text.0 = sweep_mode.label().to_string();

        // Update the existing colliders to use the new configuration.
        for entity in &balls {
            commands.entity(entity).insert(sweep_mode.ccd_settings());
        }
    }
}
