//! A simple orbital mechanics simulator.
//!
//! Notably, we use Z-up right handed.

// Recommended alias.
extern crate nalgebra as na;

/// The gravitational constant.
const _G: f64 = 6.67430e-11;

/// An approximate AU to get us going.
const _AU: f64 = 149_597_870_700.0;

fn main() {
    // Make the basic earth.
    let earth = Body::earth();
    let sun: Body = Body::sun();

    // Create a ship that is just stuck 10m in the air above the surface.
    let ship = Craft::new_above(&earth, 10.0);
    // let ship = Craft::new(
    //     na::Vector3::new(0.0, 0.0, earth.radius + 10.0), // 10 m altitude
    //     na::Vector3::new(0.0, 0.0, 0.0),                 // Stationary
    //     200.0,
    //     1.0,
    // );

    // Let's make a little force to test this.
    let thrust = Thrust {
        direction: (ship.position - earth.position).normalize(),
        magnitude: 15.0, // Newtons
        from: 0.5,       // seconds
        until: 2.0,      // seconds
    };

    let mut sim = Simulation {
        time: 0.0,
        collided: false,
        step_time: 0.01,
        print_time: 0.05,
        bodies: vec![earth, sun],
        crafts: vec![ship],
        thrust: Some(thrust),
    };

    sim.run();
    sim.show();

    println!("Final Sun position: {:?}", sim.bodies[1].position);
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
    fn earth() -> Self {
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
            khat: na::Vector3::new(
                0.0,
                f64::sin(23.5_f64.to_radians()),
                f64::cos(23.5_f64.to_radians()),
            ),
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
        let loc = na::Vector3::new(1.0, 1.0, 0.2).normalize();

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
struct Simulation {
    time: f64,
    step_time: f64,
    print_time: f64,
    collided: bool,
    bodies: Vec<Body>,
    crafts: Vec<Craft>,
    thrust: Option<Thrust>,
}

impl Simulation {
    /// Show the current position of the craft, in this case altitude and velocity.
    fn show(&self) {
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
            println!(
                "Time: {:6.3} s Altitude: {:.3} m, Speed: {:.3} m/s, hSpeed: {:.3} m/s",
                self.time, altitude, speed, hspeed,
            );
        }
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

    /// Run the simulation, stopping when the craft intersects the surface of a body.
    fn run(&mut self) {
        let mut next_print = self.time + self.print_time;
        while !self.collided {
            self.step();
            if self.time >= next_print {
                self.show();
                next_print += self.print_time;
            }
        }
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
