use std::io::{Cursor, Read, Seek, SeekFrom};

use granny2::section::{Section, SectionError};

pub mod granny2;

extern crate alloc;

pub struct Granny2 {
    pub header: granny2::Header,
    pub sections: Vec<granny2::section::Section>,
    pub root: Vec<granny2::element::Element>,
}

impl Granny2 {
    pub fn parse<T: Read + Seek>(mut reader: T) -> Result<Self, Granny2Error> {
        // Reads header
        let header = granny2::Header::parse(&mut reader)?;

        // Rewind reader back to start
        reader
            .seek(SeekFrom::Current(-(32 + i64::from(header.section_offset))))
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Seeked beyond file's beginning.",
                )
            })?;
        assert!(reader
            .stream_position()
            .ok()
            .filter(|pos| *pos == 0)
            .is_some());

        // Reads entire file into buffer
        let Ok(file_size) = usize::try_from(header.file_size) else {
            unreachable!("File size must be smaller than usize");
        };
        let input_data = {
            let mut buffer = vec![0; file_size];
            reader.read_exact(&mut buffer)?;
            buffer
        };

        // Reads all sections infos
        let Ok(section_count) = usize::try_from(header.section_count) else {
            unreachable!("Section Count must be smaller than usize.");
        };
        let sections = (0..section_count)
            .map(|section_id| -> Result<Section, SectionError> {
                let Ok(section_start) = usize::try_from(header.section_offset).map(|offset| {
                    32 + offset + section_id * size_of::<granny2::section::Section>()
                }) else {
                    unreachable!("Section Start must be smaller than usize.");
                };

                // This does not Seek internally, so we can pass just the
                // relevant data
                let mut section_raw = Cursor::new(&input_data[section_start..(section_start + 44)]);
                let section = granny2::section::Section::parse(&mut section_raw)?;
                assert_eq!(section_raw.stream_position()?, 44);

                Ok(section)
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Prepares buffer to hold decompressed data
        let Ok(decompressed_sizes) = sections
            .iter()
            .map(|section| usize::try_from(section.decompressed_size))
            .collect::<Result<Vec<_>, _>>()
        else {
            unreachable!("Total Decompressed Size must be smaller than usize.");
        };
        let mut decompressed_data = vec![0u8; decompressed_sizes.iter().sum()];

        let section_offsets = decompressed_sizes
            .iter()
            .scan(0, |accum, decompressed_size| {
                let prev = *accum;
                *accum += decompressed_size;
                Some(prev)
            })
            .collect::<Vec<_>>();

        // Read decompressed data
        for (section, (decompressed_size, offset)) in sections.iter().zip(
            decompressed_sizes
                .iter()
                .zip(section_offsets.iter().copied()),
        ) {
            // This Seek internally, so we pass the entire data
            let mut section_data = section.read_data(&mut Cursor::new(&input_data))?;

            decompressed_data[offset..(offset + decompressed_size)]
                .swap_with_slice(&mut section_data);

            #[cfg(target_endian = "big")]
            for marshalling in 0..section.marshalling_header.count {
                let Ok(pos) = usize::try_from(section.marshalling_header.offset)
                    .and_then(|offset| {
                        usize::try_from(marshalling).map(|marshalling| (offset, marshalling))
                    })
                    .map(|(offset, marshalling)| {
                        offset + marshalling * size_of::<granny2::section::Relocation>()
                    })
                else {
                    unreachable!("Marshalling position must be smaller than usize");
                };

                if pos > input_data.len() {
                    return Err(Granny2Error::from(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Marshalling would occur after end of file.",
                    )));
                }

                compile_error!("Big endian systems not supported");
            }

            for rellocation in 0..section.relocation_header.count {
                let Ok(pos) = usize::try_from(section.relocation_header.offset)
                    .and_then(|offset| {
                        usize::try_from(rellocation).map(|rellocation| (offset, rellocation))
                    })
                    .map(|(offset, rellocation)| {
                        offset + rellocation * granny2::section::Relocation::sizeof()
                    })
                else {
                    unreachable!("Rellocation position must be smaller than usize");
                };

                if pos > input_data.len() {
                    return Err(Granny2Error::from(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Relocation would occur after end of file.",
                    )));
                }

                let rellocation = granny2::section::Relocation::parse(
                    &mut &input_data[pos..(pos + granny2::section::Relocation::sizeof())],
                )?;

                let virtual_src = offset + rellocation.src_offset;
                let virtual_dst = section_offsets[rellocation.dst_section] + rellocation.dst_offset;

                decompressed_data[virtual_src..(virtual_src + 4)]
                    .swap_with_slice(&mut virtual_dst.to_le_bytes()[0..4]);
            }
        }

        let Ok(type_section) = u64::try_from(
            section_offsets[header.root_node_type.section] + header.root_node_type.offset,
        ) else {
            unreachable!("Type Section must be smaller than u64.");
        };

        let Ok(object_section) = u64::try_from(
            section_offsets[header.root_node_object.section] + header.root_node_object.offset,
        ) else {
            unreachable!("Type Section must be smaller than u64.");
        };

        let root = granny2::element::Element::parse(
            &mut Cursor::new(decompressed_data),
            type_section,
            object_section,
        )?;

        Ok(Self {
            header,
            sections,
            root,
        })
    }
}

#[derive(Debug)]
pub enum Granny2Error {
    Header,
    Section,
    Element,
    Io,
}

impl From<granny2::HeaderError> for Granny2Error {
    fn from(value: granny2::HeaderError) -> Self {
        log::error!("{}", value);
        Self::Header
    }
}

impl From<granny2::section::SectionError> for Granny2Error {
    fn from(value: granny2::section::SectionError) -> Self {
        log::error!("{}", value);
        Self::Section
    }
}

impl From<granny2::element::ElementError> for Granny2Error {
    fn from(value: granny2::element::ElementError) -> Self {
        log::error!("{}", value);
        Self::Element
    }
}

impl From<std::io::Error> for Granny2Error {
    fn from(value: std::io::Error) -> Self {
        log::error!("{}", value);
        Self::Io
    }
}
