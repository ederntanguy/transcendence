//! Implementation of the randomness and collisions needed to run a Pong game.

use std::f64::consts::{FRAC_PI_2, PI};

use rand::distributions::{Distribution, Standard, Uniform};

use crate::game::side::Side;
use crate::protocol::constants::{
    BALL_EDGE, BALL_RADIUS, HALF_BOUNCE_ANGLE_AMPL, HALF_SERVICE_ANGLE_AMPL, PAD_BOUNCE_ANGLE_AMPL,
    PAD_HEIGHT, PAD_WIDTH, RATIO,
};

/// Preemptively optimized structure containing distributions needed to generate random service angles.
#[derive(Clone)]
pub(super) struct ServiceGenerator {
    left_distribution: Uniform<f64>,
    right_side_choice_distribution: Standard,
    right_upper_angle_distribution: Uniform<f64>,
    right_lower_angle_distribution: Uniform<f64>,
}

impl ServiceGenerator {
    /// Create a new [`ServiceGenerator`] using preemptive calculating constructors of [`Distribution`]s.
    pub(super) fn new() -> ServiceGenerator {
        ServiceGenerator {
            left_distribution: Uniform::new(
                PI - HALF_SERVICE_ANGLE_AMPL,
                PI + HALF_SERVICE_ANGLE_AMPL,
            ),
            right_side_choice_distribution: Standard,
            right_upper_angle_distribution: Uniform::new(
                0.0 * PI,
                0.0 * PI + HALF_SERVICE_ANGLE_AMPL,
            ),
            right_lower_angle_distribution: Uniform::new(
                2.0 * PI - HALF_SERVICE_ANGLE_AMPL,
                2.0 * PI,
            ),
        }
    }

    /// Generates a service angle corresponding to the given [`Side`] using the given random number generator.
    pub fn gen_angle<R: rand::Rng + ?Sized>(&self, side: Side, rng: &mut R) -> f64 {
        match side {
            Side::Left => self.left_distribution.sample(rng),
            Side::Right => match self.right_side_choice_distribution.sample(rng) {
                true => self.right_upper_angle_distribution.sample(rng),
                false => self.right_lower_angle_distribution.sample(rng),
            },
        }
    }
}

/// Computes whether the ball hits one of the side walls.
/// * If so, returns the [`Side`] of the wall hit.
/// * If not, returns [`None`].
pub(super) fn side_of_ball_collision_with_wall(ball_x: f64) -> Option<Side> {
    if ball_x < 0.0 {
        Some(Side::Left)
    } else if ball_x + BALL_EDGE > RATIO {
        Some(Side::Right)
    } else {
        None
    }
}

/// Computes collisions of the ball with the top and bottom walls, and the bounce it makes. Returns the post-bounce
/// position, which can be the same as the input if no collision occurred.
pub(super) fn bounce_off_horizontal_edges(ball_y: f64, angle: f64) -> (f64, f64) {
    if ball_y <= 0.0 {
        let collision_amount = 0.0 - (ball_y + 0.0);
        (0.0 + collision_amount, 2.0 * PI - angle)
    } else if ball_y + BALL_EDGE >= 1.0 {
        let collision_amount = (ball_y + BALL_EDGE) - 1.0;
        (1.0 - collision_amount - BALL_EDGE, 2.0 * PI - angle)
    } else {
        (ball_y, angle)
    }
}

/// Computes collisions of the ball with the pads, and the bounce it makes. Returns the post-bounce position, which can
/// be the same as the input.
pub(super) fn bounce_off_pads(
    ball_x: f64,
    ball_y: f64,
    angle: f64,
    l_pad_y: f64,
    r_pad_y: f64,
) -> (f64, f64, f64) {
    if ball_pad_collide(ball_x, ball_y, 0.0, l_pad_y) {
        let collision_amount = (0.0 + PAD_WIDTH) - (ball_x + 0.0);
        let collision_x = 0.0 + PAD_WIDTH;
        let collision_y = match angle {
            angle if 1.0 * FRAC_PI_2 <= angle && angle <= 2.0 * FRAC_PI_2 => {
                ball_y + collision_amount / f64::tan(angle - FRAC_PI_2)
            }
            angle if 2.0 * FRAC_PI_2 < angle && angle <= 3.0 * FRAC_PI_2 => {
                ball_y - collision_amount * f64::tan(angle - PI)
            }
            _ => return (ball_x, ball_y, angle),
        };
        let angle = (0.0 + HALF_BOUNCE_ANGLE_AMPL) - pad_bounce_angle(collision_y, l_pad_y);
        let angle = if angle < 0.0 { angle + 2.0 * PI } else { angle };
        let correction_distance = distance(ball_x, ball_y, collision_x, collision_y);
        let ball_x = collision_x + correction_distance * f64::cos(angle);
        let ball_y = collision_y - correction_distance * f64::sin(angle);
        (ball_x, ball_y, angle)
    } else if ball_pad_collide(ball_x, ball_y, RATIO - PAD_WIDTH, r_pad_y) {
        let collision_amount = (ball_x + BALL_EDGE) - (RATIO - PAD_WIDTH);
        let collision_x = RATIO - PAD_WIDTH - BALL_EDGE;
        let collision_y = match angle {
            angle if 0.0 * FRAC_PI_2 <= angle && angle <= 1.0 * FRAC_PI_2 => {
                ball_y + collision_amount * f64::tan(angle)
            }
            angle if 3.0 * FRAC_PI_2 <= angle && angle <= 4.0 * FRAC_PI_2 => {
                ball_y - collision_amount / f64::tan(angle - 3.0 * FRAC_PI_2)
            }
            _ => return (ball_x, ball_y, angle),
        };
        let angle = (PI - HALF_BOUNCE_ANGLE_AMPL) + pad_bounce_angle(collision_y, r_pad_y);
        let correction_distance = distance(ball_x, ball_y, collision_x, collision_y);
        let ball_x = collision_x + correction_distance * f64::cos(angle);
        let ball_y = collision_y - correction_distance * f64::sin(angle);
        (ball_x, ball_y, angle)
    } else {
        (ball_x, ball_y, angle)
    }
}

fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    f64::sqrt(f64::powi(x1 - x2, 2) + f64::powi(y1 - y2, 2))
}

/// Compute whether the ball collides with the given pad.
fn ball_pad_collide(ball_x: f64, ball_y: f64, pad_x: f64, pad_y: f64) -> bool {
    if ball_y <= pad_y + PAD_HEIGHT && ball_y + BALL_EDGE >= pad_y {
        if ball_x <= pad_x + PAD_WIDTH && ball_x + BALL_EDGE >= pad_x {
            return true;
        }
    }
    return false;
}

/// Compute the angle offset based on where the ball is hitting the pad.
fn pad_bounce_angle(ball_y: f64, pad_y: f64) -> f64 {
    let ball_center_y_relative_to_pad = (ball_y + BALL_RADIUS) - pad_y;
    let amplitude_ratio = ball_center_y_relative_to_pad / PAD_HEIGHT;
    amplitude_ratio * PAD_BOUNCE_ANGLE_AMPL
}

#[cfg(test)]
mod tests {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_6};

    use rand::Rng;

    use super::*;

    const BIAS: f64 = 1.0e-7;

    #[test]
    fn service_angles() {
        let service_generator = ServiceGenerator::new();
        let mut thread_rng = rand::thread_rng();

        for angle in make_some_angles(&service_generator, Side::Left, &mut thread_rng) {
            assert!(5.0 * FRAC_PI_6 - BIAS <= angle && angle <= 7.0 * FRAC_PI_6 + BIAS);
        }
        for angle in make_some_angles(&service_generator, Side::Right, &mut thread_rng) {
            assert!(
                (0.0 * FRAC_PI_6 - BIAS <= angle && angle <= 1.0 * FRAC_PI_6 + BIAS)
                    || (11.0 * FRAC_PI_6 - BIAS <= angle && angle <= 12.0 * FRAC_PI_6 + BIAS)
            );
        }
    }

    fn make_some_angles<R: Rng + ?Sized>(
        service_generator: &ServiceGenerator,
        side: Side,
        rng: &mut R,
    ) -> Vec<f64> {
        std::iter::repeat(())
            .map(|_| service_generator.gen_angle(side, rng))
            .take(50)
            .collect()
    }

    #[test]
    fn side_collision() {
        let ball_x_too_left = 0.0 - 10.0 * BIAS;
        let ball_x_too_right = (RATIO - BALL_EDGE) + 10.0 * BIAS;
        let ball_x_valid = RATIO / 2.0;
        let ball_x_valid_rand =
            rand::thread_rng().gen_range((0.0 + BIAS)..((RATIO - BALL_EDGE) - BIAS));
        assert_eq!(
            side_of_ball_collision_with_wall(ball_x_too_left),
            Some(Side::Left)
        );
        assert_eq!(
            side_of_ball_collision_with_wall(ball_x_too_right),
            Some(Side::Right)
        );
        assert_eq!(side_of_ball_collision_with_wall(ball_x_valid), None);
        assert_eq!(side_of_ball_collision_with_wall(ball_x_valid_rand), None);
    }

    #[test]
    fn horizontal_bounces() {
        const AMOUNT: f64 = 10.0 * BIAS;

        let vertical_up_angle = FRAC_PI_2;
        let vertical_down_angle = 3.0 * FRAC_PI_2;

        let ball_y_top = 0.0 - AMOUNT;
        let ball_y_bot = (1.0 - BALL_EDGE) + AMOUNT;
        let ball_y_in_area = 1.0 / 2.0;

        let (new_y, new_angle) = bounce_off_horizontal_edges(ball_y_top, vertical_up_angle);
        assert!(0.0 + AMOUNT - BIAS <= new_y && new_y <= 0.0 + AMOUNT + BIAS);
        assert!(3.0 * FRAC_PI_2 - BIAS <= new_angle && new_angle <= 3.0 * FRAC_PI_2 + BIAS);

        let (new_y, new_angle) = bounce_off_horizontal_edges(ball_y_bot, vertical_down_angle);
        assert!(
            (1.0 - BALL_EDGE) - AMOUNT - BIAS <= new_y
                && new_y <= (1.0 - BALL_EDGE) - AMOUNT + BIAS
        );
        assert!(1.0 * FRAC_PI_2 - BIAS <= new_angle && new_angle <= 1.0 * FRAC_PI_2 + BIAS);

        let (new_y, new_angle) = bounce_off_horizontal_edges(ball_y_in_area, vertical_up_angle);
        assert_eq!(ball_y_in_area, new_y);
        assert_eq!(vertical_up_angle, new_angle);
    }
}
