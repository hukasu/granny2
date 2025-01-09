pub mod compression;
pub mod element;
pub mod reference;
pub mod section;
pub mod transform;

use std::{fmt::Display, io::Read};

use reference::Reference;

const HEADER_MAGIC: [u8; 16] = [
    184, 103, 176, 202, 248, 109, 177, 15, 132, 114, 140, 126, 94, 25, 0, 30,
];

#[derive(Debug)]
pub struct Header {
    pub magic: [u8; 16],
    pub header_size: u32,
    pub compression_type: u32,
    pub extra_bytes: [u8; 8],
    pub version: u32,
    pub file_size: u32,
    pub checksum: u32,
    pub section_offset: u32,
    pub section_count: u32,
    pub root_node_type: Reference,
    pub root_node_object: Reference,
    pub user_tag: [u8; 4],
    pub user_data: Vec<u8>,
}

impl Header {
    pub fn parse<T: Read>(reader: &mut T) -> Result<Header, HeaderError> {
        log::trace!("Parsing header");
        let magic = {
            let mut buffer = [0; 16];
            reader.read_exact(&mut buffer)?;
            buffer
        };

        if magic != HEADER_MAGIC {
            return Err(HeaderError::MagicMismatch);
        }

        let header_size = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let compression_type = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let extra_bytes = {
            let mut buffer = [0; 8];
            reader.read_exact(&mut buffer)?;
            buffer
        };

        let version = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let file_size = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let checksum = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let section_offset = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let section_count = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let root_node_type = {
            let mut section = [0; 8];
            reader.read_exact(&mut section[0..4])?;
            let mut offset = [0; 8];
            reader.read_exact(&mut offset[0..4])?;

            Reference {
                section: usize::from_le_bytes(section),
                offset: usize::from_le_bytes(offset),
            }
        };

        let root_node_object = {
            let mut section = [0; 8];
            reader.read_exact(&mut section[0..4])?;
            let mut offset = [0; 8];
            reader.read_exact(&mut offset[0..4])?;

            Reference {
                section: usize::from_le_bytes(section),
                offset: usize::from_le_bytes(offset),
            }
        };

        let user_tag = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            buffer
        };

        let user_data = {
            let Ok(section_offset) = usize::try_from(section_offset) else {
                unreachable!("Section Offset must be smaller than usize.");
            };
            let mut buffer = vec![0; section_offset - 40];
            reader.read_exact(&mut buffer)?;
            buffer
        };

        let header = Self {
            magic,
            header_size,
            compression_type,
            extra_bytes,
            version,
            file_size,
            checksum,
            section_offset,
            section_count,
            root_node_type,
            root_node_object,
            user_tag,
            user_data,
        };

        Ok(header)
    }
}

#[derive(Debug)]
pub enum HeaderError {
    OutOfBoundsRead(usize),
    Section,
    Io,
    MagicMismatch,
}

impl Display for HeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MagicMismatch => write!(f, "Header magic did not match."),
            Self::OutOfBoundsRead(section) => {
                write!(f, "Section {} would read outside of file bounds.", section)
            }
            Self::Section => write!(
                f,
                "Couldn't parse Header due to error parsing Section header."
            ),
            Self::Io => write!(f, "Couldn't parse Header due to Io error."),
        }
    }
}

impl From<std::io::Error> for HeaderError {
    fn from(value: std::io::Error) -> Self {
        log::error!("{}", value);
        Self::Io
    }
}

impl From<section::SectionError> for HeaderError {
    fn from(value: section::SectionError) -> Self {
        log::error!("{}", value);
        Self::Section
    }
}
