//! Constants corresponding to aspects of the game defined in the Protocol.

use std::f64::consts::{FRAC_PI_3, FRAC_PI_6};

pub const RATIO: f64 = 1.3;

pub const BALL_EDGE: f64 = 0.017;
/// The ball is square, so that's not really a radius.
pub const BALL_RADIUS: f64 = BALL_EDGE / 2.0;
pub const PAD_WIDTH: f64 = 0.015;
pub const PAD_HEIGHT: f64 = 0.100;

pub const BALL_MOVEMENT_PER_SECOND: f64 = 1.15;
pub const PAD_MOVEMENT_PER_SECOND: f64 = 1.5;
pub const BALL_MOVEMENT_PER_TICK: f64 = BALL_MOVEMENT_PER_SECOND / TICKS_PER_SECOND as f64;
pub const PAD_MOVEMENT_PER_TICK: f64 = PAD_MOVEMENT_PER_SECOND / TICKS_PER_SECOND as f64;

pub const MAX_SERVICE_ANGLE_AMPL: f64 = FRAC_PI_6;
pub const HALF_SERVICE_ANGLE_AMPL: f64 = MAX_SERVICE_ANGLE_AMPL / 2.0;
pub const PAD_BOUNCE_ANGLE_AMPL: f64 = FRAC_PI_3;
pub const HALF_BOUNCE_ANGLE_AMPL: f64 = FRAC_PI_6;

pub const TICKS_PER_SECOND: u64 = 100;
pub const MAX_CLIENT_UPDATES_PER_SECOND: u64 = 20;
