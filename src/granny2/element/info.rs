use std::io::{BufRead, Read, Seek};

use crate::granny2::transform::Transform;

use super::{type_id::TypeId, Data};

#[derive(Debug)]
pub struct Info {
    pub element_type: TypeId,
    name_offset: u64,
    pub children_offset: u64,
    array_size: usize,
    pub extra: [u8; 12],
    pub extra_ptr: usize,
}

impl Info {
    pub fn parse<T: Read + Seek>(
        reader: &mut T,
        types_pos: u64,
    ) -> Result<Vec<Self>, std::io::Error> {
        let rewind_pos = reader.stream_position()?;
        reader.seek(std::io::SeekFrom::Start(types_pos))?;

        let mut vec = Vec::new();

        loop {
            let element_type = {
                let mut buffer = [0; 4];
                reader.read_exact(&mut buffer)?;
                match u32::from_le_bytes(buffer).try_into() {
                    Ok(TypeId::None) | Err(_) => break,
                    Ok(other) => other,
                }
            };

            let name_offset = {
                let mut buffer = [0; 8];
                reader.read_exact(&mut buffer[0..4])?;
                u64::from_le_bytes(buffer)
            };

            let children_offset = {
                let mut buffer = [0; 8];
                reader.read_exact(&mut buffer[0..4])?;
                u64::from_le_bytes(buffer)
            };

            let array_size = {
                let mut buffer = [0; 8];
                reader.read_exact(&mut buffer[0..4])?;
                usize::from_le_bytes(buffer)
            };

            let extra = {
                let mut buffer = [0; 12];
                reader.read_exact(&mut buffer)?;
                buffer
            };

            let extra_ptr = {
                let mut buffer = [0; 8];
                reader.read_exact(&mut buffer[0..4])?;
                usize::from_le_bytes(buffer)
            };

            vec.push(Self {
                element_type,
                name_offset,
                children_offset,
                array_size,
                extra,
                extra_ptr,
            });
        }

        reader.seek(std::io::SeekFrom::Start(rewind_pos))?;

        Ok(vec)
    }

    pub fn array_size(&self) -> Option<usize> {
        match self.element_type {
            TypeId::Reference
            | TypeId::ReferenceToArray
            | TypeId::ArrayOfReferences
            | TypeId::VariantReference
            | TypeId::ReferenceToVariantArray
            | TypeId::String
            | TypeId::EmptyReference => {
                if self.has_valid_array_size() {
                    Some(1)
                } else {
                    None
                }
            }
            _ => Some(self.array_size.max(1)),
        }
    }

    pub fn read_name<T: BufRead + Seek>(&self, reader: &mut T) -> Result<Box<str>, std::io::Error> {
        if self.name_offset == 0 {
            Ok(String::new().into_boxed_str())
        } else {
            Self::read_name_from_pos(self.name_offset, reader)
        }
    }

    pub fn read_data<T: BufRead + Seek>(&self, reader: &mut T) -> Result<Data, std::io::Error> {
        let data = match self.element_type {
            TypeId::Int8 | TypeId::Int8Norm => {
                let mut buffer = [0];
                reader.read_exact(&mut buffer)?;
                Data::Int8(i8::from_le_bytes(buffer))
            }
            TypeId::UInt8 | TypeId::UInt8Norm => {
                let mut buffer = [0];
                reader.read_exact(&mut buffer)?;
                Data::UInt8(u8::from_le_bytes(buffer))
            }
            TypeId::Int16 | TypeId::Int16Norm => {
                let mut buffer = [0; 2];
                reader.read_exact(&mut buffer)?;
                Data::Int16(i16::from_le_bytes(buffer))
            }
            TypeId::UInt16 | TypeId::UInt16Norm | TypeId::Real16 => {
                let mut buffer = [0; 2];
                reader.read_exact(&mut buffer)?;
                Data::UInt16(u16::from_le_bytes(buffer))
            }
            TypeId::Int32 => {
                let mut buffer = [0; 4];
                reader.read_exact(&mut buffer)?;
                Data::Int32(i32::from_le_bytes(buffer))
            }
            TypeId::UInt32 => {
                let mut buffer = [0; 4];
                reader.read_exact(&mut buffer)?;
                Data::UInt32(u32::from_le_bytes(buffer))
            }
            TypeId::Real32 => {
                let mut buffer = [0; 4];
                reader.read_exact(&mut buffer)?;
                Data::Real32(f32::from_le_bytes(buffer))
            }
            TypeId::Transform => Data::Transform(Transform::parse(reader)?),
            TypeId::String => {
                let mut buffer = [0; 4];
                reader.read_exact(&mut buffer)?;
                let pos = u32::from_le_bytes(buffer);
                Data::String(Self::read_name_from_pos(u64::from(pos), reader)?)
            }
            TypeId::Reference
            | TypeId::ReferenceToArray
            | TypeId::VariantReference
            | TypeId::ReferenceToVariantArray
            | TypeId::EmptyReference => {
                let mut buffer = [0; 8];
                reader.read_exact(&mut buffer[..4])?;
                Data::Reference(u64::from_le_bytes(buffer))
            }
            TypeId::ArrayOfReferences => {
                let mut buffer = [0; 8];
                reader.read_exact(&mut buffer[0..4])?;
                Data::Array(u64::from_le_bytes(buffer))
            }
            TypeId::Inline | TypeId::None | TypeId::Removed => Data::Empty,
        };
        Ok(data)
    }

    fn read_name_from_pos<T: BufRead + Seek>(
        pos: u64,
        reader: &mut T,
    ) -> Result<Box<str>, std::io::Error> {
        // Get previous position
        let stream_position = reader.stream_position()?;

        // Read name
        reader.seek(std::io::SeekFrom::Start(pos)).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Failed to seek to name offset.",
            )
        })?;
        let mut buffer = vec![0; 32];
        let read = reader.read_until(0, &mut buffer).unwrap();

        // Return to previous position
        reader.seek(std::io::SeekFrom::Start(stream_position))?;

        // Parse name
        let name =
            String::from_utf8_lossy(&buffer[(buffer.len() - read)..(buffer.len() - 1)]).to_string();

        Ok(name.into_boxed_str())
    }

    /// Elements of types [`Reference`](TypeId::Reference), [`ReferenceToArray`](TypeId::ReferenceToArray),
    /// [`ArrayOfReferences`](TypeId::ArrayOfReferences), [`VariantReference`](TypeId::VariantReference),
    /// [`ReferenceToVariantArray`](TypeId::ReferenceToVariantArray), [`String`](TypeId::String),
    /// [`EmptyReference`](TypeId::EmptyReference) must have array size of 0.
    fn has_valid_array_size(&self) -> bool {
        match self.element_type {
            TypeId::Reference
            | TypeId::ReferenceToArray
            | TypeId::ArrayOfReferences
            | TypeId::VariantReference
            | TypeId::ReferenceToVariantArray
            | TypeId::String
            | TypeId::EmptyReference => self.array_size == 0,
            _ => true,
        }
    }
}
