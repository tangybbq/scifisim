//! Physics simulation library for rigid body dynamics.

extern crate nalgebra as na;
use bevy::prelude::*;

#[derive(Debug, Clone, Component)]
pub struct AttitudeState {
    /// Orientation: body -> world
    pub q_bw: na::UnitQuaternion<f64>,
    /// Angular momentum in BODY frame, half stepped for leapfrog
    pub lb_half: na::Vector3<f64>,
    /// Principal inertia in body frame.
    pub i_body: na::Vector3<f64>,
}

impl AttitudeState {
    pub fn new(
        q_bw: na::UnitQuaternion<f64>,
        lw: na::Vector3<f64>,
        i_body: na::Vector3<f64>,
    ) -> Self {
        // Convert initial world-frame L to body frame
        let lb_half = q_bw.inverse() * lw;
        Self {
            q_bw,
            lb_half,
            i_body,
        }
    }

    pub fn step(&mut self, torque_w_now: na::Vector3<f64>, dt: f64) {
        // Compute omega at half-step (L is already in body frame)
        let omega_b_half = na::Vector3::new(
            self.lb_half.x / self.i_body.x,
            self.lb_half.y / self.i_body.y,
            self.lb_half.z / self.i_body.z,
        );

        // --- DRIFT: Update orientation ---
        let dq = exp_quat(&(dt * omega_b_half));
        self.q_bw = self.q_bw * dq; // Ensure unit quaternion

        // --- KICK: Update momentum in body frame ---
        let torque_b = self.q_bw.inverse() * torque_w_now;
        let dl_dt_b = self.lb_half.cross(&omega_b_half) + torque_b;
        self.lb_half += dl_dt_b * dt;
    }

    /// Get angular momentum in world frame
    pub fn angular_momentum_world(&self) -> na::Vector3<f64> {
        self.q_bw * self.lb_half
    }
}

/// Exponential map: converts axis-angle vector to unit quaternion.
///
/// Given a 3D vector v = θ * n (where n is unit axis, θ is rotation angle),
/// returns the unit quaternion q representing rotation by θ radians around n.
fn exp_quat(v: &na::Vector3<f64>) -> na::UnitQuaternion<f64> {
    let theta = v.norm();
    if theta < 1e-10 {
        // No significant rotation
        na::UnitQuaternion::identity()
    } else {
        na::UnitQuaternion::from_axis_angle(&na::Unit::new_normalize(*v), theta)
    }
}
