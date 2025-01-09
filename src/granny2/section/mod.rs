mod compression_mode;
mod marshalling;
mod marshalling_header;
mod relocation;
mod relocation_header;

use std::{
    fmt::Display,
    io::{Read, Seek},
};

pub use self::{
    compression_mode::{CompressionMode, CompressionModeError},
    marshalling_header::MarshallingHeader,
    relocation::Relocation,
    relocation_header::RelocationHeader,
};

use super::compression::{Oodle, OodleError};

#[derive(Debug)]
pub struct Section {
    pub compression_mode: CompressionMode,
    pub section_offset: u32,
    pub compressed_size: u32,
    pub decompressed_size: u32,
    pub alignment_size: u32,
    pub stop_0: u32,
    pub stop_1: u32,
    pub relocation_header: relocation_header::RelocationHeader,
    pub marshalling_header: marshalling_header::MarshallingHeader,
}

impl Section {
    pub fn parse<T: Read>(mut reader: &mut T) -> Result<Self, SectionError> {
        log::trace!("Parsing section header");
        let compression_mode = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer).try_into()?
        };

        let section_offset = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let compressed_size = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let decompressed_size = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        if compression_mode == CompressionMode::None && compressed_size != decompressed_size {
            return Err(SectionError::NoCompressionSizeMismatch(
                compressed_size,
                decompressed_size,
            ));
        }

        let alignment_size = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let stop_0 = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let stop_1 = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let relocation_header = relocation_header::RelocationHeader::parse(&mut reader)?;

        let marshalling_header = marshalling_header::MarshallingHeader::parse(&mut reader)?;

        let header = Self {
            compression_mode,
            section_offset,
            compressed_size,
            decompressed_size,
            alignment_size,
            stop_0,
            stop_1,
            relocation_header,
            marshalling_header,
        };

        Ok(header)
    }

    pub fn read_data<T: Read + Seek>(&self, reader: &mut T) -> Result<Vec<u8>, SectionError> {
        reader.seek(std::io::SeekFrom::Start(u64::from(self.section_offset)))?;
        if self.compression_mode == CompressionMode::None {
            let Ok(decompressed_size) = usize::try_from(self.decompressed_size) else {
                return Err(SectionError::BufferCreation(self.decompressed_size));
            };

            let mut data = vec![0; decompressed_size];
            reader.read_exact(&mut data)?;
            Ok(data)
        } else {
            let Ok(compressed_size) = usize::try_from(self.compressed_size) else {
                return Err(SectionError::BufferCreation(self.compressed_size));
            };
            let Ok(decompressed_size) = usize::try_from(self.decompressed_size) else {
                return Err(SectionError::BufferCreation(self.decompressed_size));
            };

            match self.compression_mode {
                CompressionMode::Oodle0 | CompressionMode::Oodle1 => {
                    let Ok(stop_0) = usize::try_from(self.stop_0) else {
                        return Err(SectionError::BufferCreation(self.stop_0));
                    };
                    let Ok(stop_1) = usize::try_from(self.stop_1) else {
                        return Err(SectionError::BufferCreation(self.stop_1));
                    };
                    Oodle::decompress(reader, compressed_size, decompressed_size, stop_0, stop_1)
                        .map_err(SectionError::from)
                }
                CompressionMode::Bitknit1 | CompressionMode::Bitknit2 => {
                    Ok(vec![0; compressed_size])
                }
                CompressionMode::None => unreachable!("CompressionMode None already dealt with."),
            }
        }
    }
}

#[derive(Debug)]
pub enum SectionError {
    BufferCreation(u32),
    NoCompressionSizeMismatch(u32, u32),
    CompressionMode,
    Io,
}

impl Display for SectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SectionError::BufferCreation(size) => {
                write!(f, "Can't create buffer of size {}.", size)
            }
            Self::NoCompressionSizeMismatch(compressed_size, decompressed_size) => write!(
                f,
                "Compressed size and decompressed size differ on Compression Mode 0. ({} != {})",
                compressed_size, decompressed_size
            ),
            Self::CompressionMode => write!(f, "Section had invalid compression mode."),
            Self::Io => write!(f, "Couldn't parse Section due to Io error."),
        }
    }
}

impl From<std::io::Error> for SectionError {
    fn from(value: std::io::Error) -> Self {
        log::error!("{}", value);
        Self::Io
    }
}

impl From<CompressionModeError> for SectionError {
    fn from(value: CompressionModeError) -> Self {
        log::error!("{}", value);
        Self::CompressionMode
    }
}

impl From<OodleError> for SectionError {
    fn from(value: OodleError) -> Self {
        log::error!("{}", value);
        Self::CompressionMode
    }
}
