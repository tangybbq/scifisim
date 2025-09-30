//! A simple orbital mechanics simulator.
//!
//! Notably, we use Z-up right handed.

// Recommended alias.
extern crate nalgebra as na;

mod solar;

use std::io::Write;

use bevy::{
    color::palettes::css::{GOLD, GREEN},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

/// The gravitational constant.
const _G: f64 = 6.67430e-11;

/// An approximate AU to get us going.
const _AU: f64 = 149_597_870_700.0;

fn main() {
    solar::init_spice();
    if true {
        return;
    }
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin::default()));
    app.add_systems(Startup, setup);
    app.add_systems(Update, text_update_system);
    app.add_systems(Update, text_update_fps);
    app.add_systems(Update, keyboard_input_system);

    setup_sim(&mut app);
    app.run();
}

#[derive(Resource)]
struct Paused(bool);

// Marker struct to identify the main camera.
#[derive(Component)]
struct MainCamera;

// Marker struct to identify the text component.
#[derive(Component)]
struct StateText;

// Marker for bodies, gives index into `body` in the sim.
#[derive(Component)]
struct BodyIndex(usize);

// Marker for crafts, gives index into `craft` in the sim.
#[derive(Component)]
struct CraftIndex(usize);

#[derive(Component)]
struct FpsText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Paused(false));

    commands.spawn((
        Text::new(""),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            font_size: 24.0,
            ..default()
        },
        TextShadow::default(),
        TextLayout::new_with_justify(JustifyText::Left),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
        StateText,
    ));

    // Framerate display
    commands
        .spawn((
            Text::new("FPS: "),
            TextFont {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 24.0,
                ..default()
            },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                right: Val::Px(5.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextFont {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 24.0,
                ..default()
            },
            TextColor(GOLD.into()),
            FpsText,
        ));

    // Create the main 3d camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_3,
            near: 0.1,
            far: 1.0e12,
            ..default()
        }),
        MainCamera,
    ));

    // Bring in the ship.
    commands.spawn((
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/output.gltf")),
            // Make it slowly rotate.
            // Rotate { axis: Vec3::Y, speed: 0.1 },
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
        CraftIndex(0),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.6,
        ..Default::default()
    });

    commands.insert_resource(DirectionalLightShadowMap {
        size: 8192,
        ..default()
    });

    // The sun.
    commands.spawn((
        DirectionalLight {
            illuminance: 30_000.0,
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 10.0e8,
            maximum_distance: 100.0e8,
            ..default()
        }
        .build(),
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            std::f32::consts::PI,
            0.0,
            0.0,
        )),
    ));

    let earth_mesh = Sphere { radius: 6.371e6 }.mesh().ico(40).unwrap();
    let earth = meshes.add(earth_mesh);
    let grass = materials.add(StandardMaterial {
        base_color: GREEN.into(),
        ..default()
    });

    // A sphere for the earth.
    commands.spawn((
        Mesh3d(earth),
        MeshMaterial3d(grass),
        Transform::from_xyz(0.0, -6.371e6 - 10.0, 0.0),
        BodyIndex(0),
    ));

    // Create some objects "quads" in the distance.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))), // Red
        Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_8)
                * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        ),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))), // Green
        Transform::from_xyz(0.0, 0.0, -0.001).with_rotation(
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_8)
                * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        ),
    ));
}

fn keyboard_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<Paused>,
    mut time: ResMut<Time<Virtual>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        paused.0 = !paused.0;

        if paused.0 {
            time.pause();
        } else {
            time.unpause();
            // time.set_relative_speed(60.0);
        }
    }
}

fn text_update_system(
    time: Res<Time<Virtual>>,
    mut sim: ResMut<Simulation>,
    mut query: Query<&mut Text, With<StateText>>,
    paused: Res<Paused>,
    mut bodies: Query<(&BodyIndex, &mut Transform), (With<BodyIndex>, Without<CraftIndex>)>,
    mut crafts: Query<(&CraftIndex, &mut Transform), (With<CraftIndex>, Without<BodyIndex>)>,
    mut cameras: Query<
        &mut Transform,
        (With<MainCamera>, (Without<CraftIndex>, Without<BodyIndex>)),
    >,
) {
    // println!("text update {:.4}", time.elapsed_secs());

    // Run the simulation until it's time reaches our current time.
    let seconds: f32 = time.elapsed_secs();
    let mut count = 0;
    while sim.time < seconds as f64 && !sim.collided {
        // println!("  stepping sim at {:.3}", sim.time);
        sim.step();
        count += 1;
    }

    if let Ok(mut text) = query.single_mut() {
        let mut message = Vec::new();
        if paused.0 {
            writeln!(message, "Paused").unwrap();
        } else {
            writeln!(message, "Running").unwrap();
        }
        writeln!(message, "Simulation time: {:.3} seconds", seconds).unwrap();
        sim.write(&mut message);
        writeln!(message, "{} physics steps", count).unwrap();
        **text = String::from_utf8(message).unwrap();
        // **text = format!("Some text now: {:.2} seconds", seconds);
        // println!("  subtext: {}", **text);
    }

    let mut camera = cameras.single_mut().unwrap();

    // Update the crafts as needed.
    for (index, mut position) in &mut crafts {
        // For now, only update the first craft.
        if index.0 != 0 {
            continue;
        }

        // Craft rel planet.
        let p_sim = sim.crafts[index.0].position - sim.bodies[0].position;
        let v_sim = sim.crafts[index.0].velocity - sim.bodies[0].velocity;

        // Tangent-frame basis in SIM space.
        let up_sim = p_sim.normalize();

        let vh_sim = {
            let proj = v_sim.dot(&up_sim);
            v_sim - up_sim * proj
        };
        let vhat_sim = if vh_sim.norm_squared() > 1e-12 {
            vh_sim.normalize()
        } else {
            // If nearly vertical, just project a north.
            let north = na::Vector3::new(0.0, 1.0, 0.0);
            let nproj = north - up_sim * north.dot(&up_sim);
            if nproj.norm_squared() > 0.0 {
                nproj.normalize()
            } else {
                // If we are exactly at the pole, just pick something.
                na::Vector3::x_axis().into_inner()
            }
        };

        // right
        let xhat_sim = up_sim.cross(&vhat_sim).normalize();
        // forward (along track)
        let zhat_sim = xhat_sim.cross(&up_sim).normalize();

        // Camera offset in Sim space.
        let back = 10.0;
        // let back = 1.0e6;
        let up_h = 2.0;
        let cam_offset_sim = -back * zhat_sim + up_h * up_sim;

        let up_b = sim_to_bevy(&up_sim);
        // let zhat_b = sim_to_bevy(&zhat_sim);
        let craft_pos_b = position.translation;
        let cam_pos_b = craft_pos_b + sim_to_bevy(&cam_offset_sim);
        // println!("Camera pos: {:?}", cam_pos_b);

        camera.translation = cam_pos_b;
        // camera.look_to(-zhat_b, up_b);
        camera.look_at(craft_pos_b, up_b);

        // Rotate the craft to match "up".  This is not actually correct, and it
        // should be rotated according to the thrust vector, or it's own
        // internal orientation, but we don't store that yet.
        position.rotation = Quat::from_rotation_arc(Vec3::Y, up_b);
    }

    // Update the bodies.
    for (index, mut position) in &mut bodies {
        // The user's craft is the first craft.
        let pos = sim.bodies[index.0].position - sim.crafts[0].position;
        position.translation = sim_to_bevy(&pos);
        // println!(
        //     "Body {} position: {:?}, dist: {:04e}",
        //     index.0,
        //     pos,
        //     position.translation.norm()
        // );
    }
}

/// Convert a sim vector to a Bevy vector.
fn sim_to_bevy(v: &na::Vector3<f64>) -> Vec3 {
    Vec3::new(v.x as f32, v.z as f32, -v.y as f32)
}

fn text_update_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    if let Ok(mut text) = query.single_mut() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                **text = format!("{value:.2}");
            }
        }
    }
}

fn setup_sim(app: &mut App) {
    // Make the basic earth.
    let earth = Body::earth();
    let sun: Body = Body::sun();

    // Create a ship that is just stuck 1km in the air above the surface.
    let ship = Craft::new_above(&earth, 100.0);

    // Let's make a little force to test this.
    let thrust = Thrust {
        direction: (ship.position - earth.position).normalize(),
        magnitude: 15.0, // Newtons
        from: 0.5,       // seconds
        until: 2.0,      // seconds
    };

    let sim = Simulation {
        time: 0.0,
        collided: false,
        step_time: 1.0 / 100.0,
        bodies: vec![earth, sun],
        crafts: vec![ship],
        thrust: Some(thrust),
    };

    app.insert_resource(sim);
}

/// A "large" object in space.  This represents planets, and anything that has a
/// significant gravitational effect on the simulation. Significant, in this
/// case being that the magnitude of force other objects experience from it
/// exceed the floating point precision of the other forces involved.
#[derive(Debug)]
struct Body {
    position: na::Vector3<f64>,
    velocity: na::Vector3<f64>,
    mu: f64,
    radius: f64,
    khat: na::Vector3<f64>, // The direction of the north pole.
    omega: f64,             // The angular velocity of the body, in radians per second.
}

impl Body {
    /// Create a Body representing earth, with a reference frame centered on it.
    /// This is not actually correct, as the rest of the simulation assumes a
    /// non-rotating reference frame, so this is only temporary.
    ///
    /// Note that these possitions are not consistent.  J2000 defines the
    /// coordinate system as of Jan 1, 2000, but the ephemeris data is for Sep
    /// 23, 2025.  The axial tilt is also for J2000, which is not quite the
    /// same.  Also, the axial tilt is directly along the Y axis, but only at
    /// J2000.
    fn earth() -> Self {
        // Earth's axial tilt, as of J2000.
        const AXIAL: f64 = 23.43928f64.to_radians();

        // Let's start the earth at an unusual posisition, just picking
        // somewhere so that both values are not near zero.
        // let x = f64::cos(23.5_f64.to_radians()) * AU;
        // let y = f64::sin(23.5_f64.to_radians()) * AU;
        Body {
            // This is from the NASA ephemeris for the time below
            // 2460941.500000000 = A.D. 2025-Sep-23 00:00:00.0000 TDB
            //  X = 1.495620660480920E+08 Y =-1.147519768700426E+06 Z = 2.115514734450541E+04
            //  VX=-4.082628156136917E-01 VY= 2.968689110543276E+01 VZ=-9.955089786526372E-04
            //  LT= 4.989000412766351E+02 RG= 1.495664696706239E+08 RR=-6.360178580009550E-01
            position: na::Vector3::new(
                1.495620660480920E+08 * 1.0e3,
                -1.147519768700426E+06 * 1.0e3,
                2.115514734450541E+04 * 1.0e3,
            ),
            velocity: na::Vector3::new(
                -4.082628156136917E-01 * 1.0e3,
                2.968689110543276E+01 * 1.0e3,
                -9.955089786526372E-04 * 1.0e3,
            ),
            mu: 3.986004418e14,
            radius: 6.371e6, // Average radius in meters.
            khat: na::Vector3::new(0.0, f64::sin(AXIAL), f64::cos(AXIAL)),
            omega: 2.0 * std::f64::consts::PI / 86164.0, // One rotation per sideral day.
        }
    }

    /// The sun is fairly simple.
    fn sun() -> Self {
        Body {
            position: na::Vector3::new(0.0, 0.0, 0.0),
            velocity: na::Vector3::new(0.0, 0.0, 0.0),
            mu: 1.32712440018e20,
            radius: 6.9634e8, // Average radius in meters.
            khat: na::Vector3::new(0.0, 0.0, 1.0),
            omega: 0.0, // Neglecting rotation for now.
        }
    }
}

/// A "small" object in space.  This represents things like spacecraft, and
/// asteroids.  These objects are affected by the gravity of large bodies, but
/// do not themselves exert a significant gravitational force on other objects.
#[derive(Debug)]
struct Craft {
    position: na::Vector3<f64>,
    velocity: na::Vector3<f64>,
    #[allow(dead_code)]
    mass: f64,
    // Simple spherical collision model.
    radius: f64,
}

impl Craft {
    /// Create a new craft, above the given Body, initially, not moving.
    /// This basically assumes we are above the ecliptic north pole, which is very much not reasonable.
    fn new_above(body: &Body, altitude: f64) -> Self {
        let loc = na::Vector3::new(-1.0, 2.0, 1.2).normalize();

        let position = body.position + loc * (body.radius + altitude);
        let rel_pos = position - body.position;
        let big_omega = body.omega * body.khat;
        let surface_speed = big_omega.cross(&rel_pos);
        let velocity = body.velocity + surface_speed;
        Craft {
            position,
            velocity,
            mass: 200.0,
            radius: 1.0,
        }
    }

    #[allow(dead_code)]
    fn new(position: na::Vector3<f64>, velocity: na::Vector3<f64>, mass: f64, radius: f64) -> Self {
        Craft {
            position,
            velocity,
            mass,
            radius,
        }
    }
}

/// A simulation of bodies and crafts in space.
#[derive(Resource)]
struct Simulation {
    time: f64,
    step_time: f64,
    collided: bool,
    bodies: Vec<Body>,
    crafts: Vec<Craft>,
    thrust: Option<Thrust>,
}

impl Simulation {
    /// Show the current position of the craft, in this case altitude and velocity.
    fn write(&self, mut out: impl Write) {
        for craft in &self.crafts {
            // Assume the first body is the central body.
            let body = &self.bodies[0];
            let rel_pos = craft.position - body.position;
            let up = rel_pos.normalize();
            let altitude = rel_pos.norm() - body.radius;
            let rel_vel = craft.velocity - body.velocity;
            let speed = rel_vel.dot(&up);
            let big_omega = body.omega * body.khat;
            let ground_speed = big_omega.cross(&(body.radius * up));

            // Calculate hspeed based on ground speed.
            let hspeed = (rel_vel - up * speed - ground_speed).norm();

            // println!("Ground speed: {:?}", ground_speed);
            // let speed = rel_vel.norm();
            writeln!(
                out,
                "Time: {:6.3} s Altitude: {:.3} m, Speed: {:.3} m/s, hSpeed: {:.3} m/s",
                self.time, altitude, speed, hspeed,
            )
            .unwrap();
        }
    }

    #[allow(dead_code)]
    fn show(&self) {
        self.write(std::io::stdout());
    }

    /// Step the simulation forward by the given time step, in seconds.
    fn step(&mut self) {
        // Update the position and velocity of each craft.
        let mut first = true;
        for craft in &mut self.crafts {
            // Calculate the total acceleration on the craft due to all bodies.
            let mut total_acceleration = na::Vector3::new(0.0, 0.0, 0.0);
            for body in &self.bodies {
                let rel_pos = body.position - craft.position;
                let distance = rel_pos.norm();
                if distance < body.radius + craft.radius {
                    self.collided = true;
                    println!("Impact detected!");
                    continue;
                }
                let acceleration = rel_pos * body.mu / (distance * distance * distance);
                total_acceleration += acceleration;
            }

            if first {
                first = false;
                // Apply thrust if we have it, and it's active.
                if let Some(thrust) = &self.thrust {
                    if thrust.is_active(self.time) {
                        let thrust_accel = thrust.force();
                        total_acceleration += thrust_accel;
                    }
                }
            }

            // Update velocity and position using simple Euler integration.
            craft.velocity += total_acceleration * self.step_time;
            craft.position += craft.velocity * self.step_time;
        }

        // Update the position of each body as well.
        let mut accels = Vec::new();
        for body in &self.bodies {
            let mut acceleration = na::Vector3::new(0.0, 0.0, 0.0);
            for other in &self.bodies {
                if std::ptr::eq(body, other) {
                    continue;
                }
                let rel_pos = other.position - body.position;
                let distance = rel_pos.norm();
                acceleration += rel_pos * other.mu / (distance * distance * distance);
            }
            accels.push(acceleration);
        }

        // Apply the accumulated accelerations to each body.
        for (body, accel) in self.bodies.iter_mut().zip(accels.iter()) {
            body.velocity += *accel * self.step_time;
            body.position += body.velocity * self.step_time;
        }

        self.time += self.step_time;
    }
}

struct Thrust {
    direction: na::Vector3<f64>,
    magnitude: f64,
    from: f64,
    until: f64,
}

impl Thrust {
    #[allow(dead_code)]
    fn is_active(&self, time: f64) -> bool {
        time >= self.from && time <= self.until
    }

    #[allow(dead_code)]
    fn force(&self) -> na::Vector3<f64> {
        self.direction.normalize() * (self.magnitude)
    }
}
