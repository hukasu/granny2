use std::{error::Error, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMode {
    None,
    Oodle0,
    Oodle1,
    Bitknit1,
    Bitknit2,
}

impl TryFrom<u32> for CompressionMode {
    type Error = CompressionModeError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Oodle0),
            2 => Ok(Self::Oodle1),
            3 => Ok(Self::Bitknit1),
            4 => Ok(Self::Bitknit2),
            other => Err(CompressionModeError(other)),
        }
    }
}

#[derive(Debug)]
pub struct CompressionModeError(pub u32);

impl Display for CompressionModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid Compression Mode {}.", self.0)
    }
}

impl Error for CompressionModeError {}
