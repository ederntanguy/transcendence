//! Protocol-compliant serialization/deserialization for [`Side`].

use crate::game::Side;

/// Errors encountered when making a [`Side`] out of a [`u8`].
#[derive(thiserror::Error, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub enum SideCastError {
    #[error("A Side is either 0 for Left or 1 for Right - got `{0}`")]
    InvalidInteger(u8),
}

impl TryFrom<u8> for Side {
    type Error = SideCastError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Right),
            n => Err(Self::Error::InvalidInteger(n)),
        }
    }
}

impl From<Side> for u8 {
    fn from(value: Side) -> Self {
        match value {
            Side::Left => 0,
            Side::Right => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn side_to_u8() {
        assert_eq!(u8::from(Side::Left), 0u8);
        assert_eq!(u8::from(Side::Right), 1u8);
    }

    #[test]
    fn u8_to_side() {
        // Ok
        assert_eq!(Side::try_from(0u8), Ok(Side::Left));
        assert_eq!(Side::try_from(1u8), Ok(Side::Right));

        // Err
        assert_eq!(Side::try_from(5u8), Err(SideCastError::InvalidInteger(5u8)));
        let invalid_u8 = rand::thread_rng().gen_range(2u8..=u8::MAX);
        assert_eq!(
            Side::try_from(invalid_u8),
            Err(SideCastError::InvalidInteger(invalid_u8))
        );
    }
}
