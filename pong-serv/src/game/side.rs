//! Definition of the [`Side`] structure.

use std::ops::Not;

use rand::distributions::{Distribution, Standard};
use rand::Rng;

/// Enumeration symbolizing sides : left or right.
///
/// The [`Not`] trait is implemented to support inversion using `!s` syntax.
///
/// An implementation of [`Distribution`] of [`Side`]s for [`Standard`] is given to make it easy to generate random
/// sides.
///
/// Conversions to and from [`u8`] are implemented in [`crate::protocol`]. They follow the Protocol.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Side {
    Left,
    Right,
}

impl Not for Side {
    type Output = Side;
    fn not(self) -> Self::Output {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl Distribution<Side> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Side {
        match rng.gen() {
            true => Side::Left,
            false => Side::Right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_inversion() {
        assert_eq!(!Side::Left, Side::Right);
        assert_eq!(!Side::Right, Side::Left);
    }
}
