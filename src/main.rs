//! A simple orbital mechanics simulator.
//!
//! Notably, we use Z-up right handed.

// Recommended alias.
extern crate nalgebra as na;

/// The gravitational constant.
const _G: f64 = 6.67430e-11;

fn main() {
    // Make the basic earth.
    let earth = Body::earth();

    // Create a ship that is just stuck 10m in the air above the surface.
    let ship = Craft::new(
        na::Vector3::new(0.0, 0.0, earth.radius + 10.0), // 10 m altitude
        na::Vector3::new(0.0, 0.0, 0.0),                 // Stationary
        200.0,
        1.0,
    );

    let mut sim = Simulation {
        time: 0.0,
        collided: false,
        step_time: 0.01,
        print_time: 0.05,
        bodies: vec![earth],
        crafts: vec![ship],
    };

    sim.show();
    sim.run();
    sim.show();
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
}

impl Body {
    /// Create a Body representing earth, with a reference frame centered on it.
    /// This is not actually correct, as the rest of the simulation assumes a
    /// non-rotating reference frame, so this is only temporary.
    fn earth() -> Self {
        Body {
            position: na::Vector3::new(0.0, 0.0, 0.0),
            velocity: na::Vector3::new(0.0, 0.0, 0.0),
            // mu: G * 5.972e24,
            mu: 3.986004418e14,
            radius: 6.371e6, // Average radius in meters.
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
    /// Create a new craft at the given position and velocity.
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
}

impl Simulation {
    /// Show the current position of the craft, in this case altitude and velocity.
    fn show(&self) {
        for craft in &self.crafts {
            // Assume the first body is the central body.
            let body = &self.bodies[0];
            let rel_pos = craft.position - body.position;
            let altitude = rel_pos.norm() - body.radius;
            let rel_vel = craft.velocity - body.velocity;
            let speed = rel_vel.norm();
            println!("Altitude: {:.3} m, Speed: {:.3} m/s", altitude, speed);
        }
    }

    /// Step the simulation forward by the given time step, in seconds.
    fn step(&mut self) {
        // Update the position and velocity of each craft.
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
                // let acceleration = rel_pos.normalize() * acceleration_magnitude;
                total_acceleration += acceleration;
            }

            // Update velocity and position using simple Euler integration.
            craft.velocity += total_acceleration * self.step_time;
            craft.position += craft.velocity * self.step_time;
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
