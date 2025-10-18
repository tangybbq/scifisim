//! Physics simulation library for rigid body dynamics.

extern crate nalgebra as na;
use bevy::prelude::*;

/// Attitude (rotation) state advanced with the improved PCDM leapfrog scheme:
/// q (body->world) and ω live at half-steps; r lives at whole steps.
#[derive(Debug, Clone, Component)]
pub struct AttitudeState {
    /// Orientation BODY -> WORLD at half-step (n + 1/2).
    pub q_bw: na::UnitQuaternion<f64>,

    /// Angular velocity in BODY frame at half-step: ω_b(n + 1/2).
    pub omega_b_half: na::Vector3<f64>,

    /// Principal moments of inertia in BODY frame (diagonal): (I_x, I_y, I_z).
    pub i_body: na::Vector3<f64>,

    /// Cached angular acceleration in BODY at the previous whole step: ω̇_b(n).
    pub omega_dot_b_prev: na::Vector3<f64>,
}

impl AttitudeState {
    /// Construct from q(n+1/2), ω_b(n+1/2), I_body, and τ_b(n).
    /// We compute ω̇_b(n) = I^{-1} ( τ_b(n) - ω_b(n+1/2) × (I ω_b(n+1/2)) ).
    pub fn new_with_omega_b(
        q_half: na::UnitQuaternion<f64>,
        omega_b_half: na::Vector3<f64>,
        i_body: na::Vector3<f64>,
        tau_b_at_n: na::Vector3<f64>,
    ) -> Self {
        let omega_dot_b_prev = Self::omega_dot_b_static(&i_body, &omega_b_half, &tau_b_at_n);
        Self {
            q_bw: q_half,
            omega_b_half,
            i_body,
            omega_dot_b_prev,
        }
    }

    /// Convenience: transform ω_b(n+1/2) to WORLD.
    pub fn omega_world_half(&self) -> na::Vector3<f64> {
        self.q_bw.transform_vector(&self.omega_b_half)
    }

    /// I ∘ ω  (component-wise since I is diagonal in BODY)
    #[inline]
    fn i_mul(&self, omega_b: &na::Vector3<f64>) -> na::Vector3<f64> {
        self.i_body.component_mul(omega_b)
    }

    /// I^{-1} ∘ v  (component-wise since I is diagonal in BODY)
    #[inline]
    fn i_inv_mul(&self, v_b: &na::Vector3<f64>) -> na::Vector3<f64> {
        v_b.component_div(&self.i_body)
    }

    /// Static helper: ω̇_b = I^{-1} ( τ_b - ω_b × (I ω_b) )
    #[inline]
    fn omega_dot_b_static(
        i_body: &na::Vector3<f64>,
        omega_b: &na::Vector3<f64>,
        tau_b: &na::Vector3<f64>,
    ) -> na::Vector3<f64> {
        let i_omega = i_body.component_mul(omega_b);
        let coriolis = omega_b.cross(&i_omega);
        (tau_b - coriolis).component_div(i_body)
    }

    /// Improved PCDM (rotational-only) step with *precomputed* body-frame torque at n+1.
    ///
    /// Inputs:
    ///   - dt: step size
    ///   - tau_b_n1: τ_b(n+1), body-frame torque you computed before calling step
    ///
    /// Updates internal state to (q, ω_b) at (n+3/2) and caches ω̇_b(n+1).
    /// Returns (q_{n+3/2}, ω_world_{n+3/2}) for convenience.
    pub fn step_rot_fixed_tau_b(
        &mut self,
        dt: f64,
        tau_b_n1: na::Vector3<f64>,
    ) -> (na::UnitQuaternion<f64>, na::Vector3<f64>) {
        // ---- Step 2(b): predict ω_b(3/4) and q'(n+1) ----
        // ω_b(3/4) = ω_b(n+1/2) + 0.25 * ω̇_b(n) * dt
        let omega_b_three_quarters = self.omega_b_half + 0.25 * self.omega_dot_b_prev * dt;

        // ω_lab(3/4) via q(n+1/2)
        let omega_lab_three_quarters = self.q_bw.transform_vector(&omega_b_three_quarters);

        // q'(n+1) = exp( ω_lab(3/4) * (dt/2) ) * q(n+1/2)
        let dq_half = na::UnitQuaternion::from_scaled_axis(omega_lab_three_quarters * (0.5 * dt));
        let q_pred_n1 = dq_half * self.q_bw;

        // ---- Step 2(c): predict ω'_b(n+1) and ω'_lab(n+1) ----
        // ω'_b(n+1) = ω_b(n+1/2) + 0.5 * ω̇_b(n) * dt
        let omega_b_pred_n1 = self.omega_b_half + 0.5 * self.omega_dot_b_prev * dt;
        let omega_lab_pred_n1 = q_pred_n1.transform_vector(&omega_b_pred_n1);

        // ---- Step 3: use τ_b(n+1) to compute ω̇_b(n+1) ----
        // ω̇_b(n+1) = I^{-1} ( τ_b(n+1) - ω'_b(n+1) × (I ω'_b(n+1)) )
        let i_omega_pred = self.i_mul(&omega_b_pred_n1);
        let coriolis_pred = omega_b_pred_n1.cross(&i_omega_pred);
        let omega_dot_b_n1 = self.i_inv_mul(&(tau_b_n1 - coriolis_pred));

        // ---- Step 4: correct to n+3/2 ----
        // ω_b(n+3/2) = ω_b(n+1/2) + ω̇_b(n+1) * dt
        let omega_b_next_half = self.omega_b_half + omega_dot_b_n1 * dt;

        // q(n+3/2) = exp( ω'_lab(n+1) * dt ) * q(n+1/2)
        let dq_full = na::UnitQuaternion::from_scaled_axis(omega_lab_pred_n1 * dt);
        let q_next_half = dq_full * self.q_bw;

        // ω_lab(n+3/2)
        let omega_lab_next_half = q_next_half.transform_vector(&omega_b_next_half);

        // ---- Commit for next iteration ----
        self.q_bw = q_next_half;
        self.omega_b_half = omega_b_next_half;
        self.omega_dot_b_prev = omega_dot_b_n1;

        (q_next_half, omega_lab_next_half)
    }
}
