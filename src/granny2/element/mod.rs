mod data;
mod info;
mod type_id;

use std::{
    error::Error,
    fmt::Display,
    io::{BufRead, Seek},
};

pub use self::{
    data::Data,
    info::{Info, InfoError},
    type_id::TypeId,
};

#[derive(Debug)]
pub struct Element {
    pub info: Info,
    pub name: Box<str>,
    pub children: Vec<Element>,
    pub size: usize,
    pub data: Vec<Data>,
}

impl Element {
    pub fn parse<T: BufRead + Seek>(
        reader: &mut T,
        types_pos: u64,
        object_pos: u64,
    ) -> Result<Vec<Self>, ElementError> {
        let rewind_pos = reader.stream_position()?;

        reader.seek(std::io::SeekFrom::Start(object_pos))?;
        let mut elements = Vec::new();
        for type_info in Info::parse(reader, types_pos)? {
            elements.push(Element::parse_single(reader, type_info)?);
        }

        reader.seek(std::io::SeekFrom::Start(rewind_pos))?;
        Ok(elements)
    }

    pub fn parse_single<T: BufRead + Seek>(
        reader: &mut T,
        info: Info,
    ) -> Result<Self, ElementError> {
        let name = info.read_name(reader)?;

        let size = info.array_size;

        let data = info.read_data(reader)?;

        let children = Self::read_children(reader, &info, &data)?;

        Ok(Element {
            info,
            name,
            children,
            size,
            data,
        })
    }

    fn read_children<T: BufRead + Seek>(
        reader: &mut T,
        info: &Info,
        data: &[Data],
    ) -> Result<Vec<Element>, ElementError> {
        let rewind_pos = reader.stream_position()?;

        let children = match (info.element_type, data) {
            (TypeId::Reference | TypeId::EmptyReference, [Data::Reference(ref_pos)])
                if *ref_pos != 0 =>
            {
                Self::parse(reader, info.children_offset, *ref_pos)?
            }
            (TypeId::ArrayOfReferences, [Data::ArrayOfReferences(references)]) => {
                let mut children = vec![];

                for reference in references {
                    children.extend(Self::parse(reader, info.children_offset, *reference)?);
                }

                children
            }
            // (TypeId::ReferenceToArray, Data::Array(size, pos)) if *size != 0 => {
            //     Self::parse(reader, info.children_offset, *pos)?
            // }
            (TypeId::Inline, [Data::Empty]) => {
                Self::parse(reader, info.children_offset, rewind_pos)?
            }
            _ => vec![],
        };

        reader.seek(std::io::SeekFrom::Start(rewind_pos))?;

        Ok(children)
    }
}

#[derive(Debug)]
pub enum ElementError {
    InvalidType,
    Info,
    Io,
}

impl Display for ElementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidType => write!(f, "Couldn't create an element from type info."),
            Self::Info => write!(f, "An error occurred while reading Element's Info."),
            Self::Io => write!(f, "Couldn't parse elements due to Io error."),
        }
    }
}

impl Error for ElementError {}

impl From<std::io::Error> for ElementError {
    fn from(value: std::io::Error) -> Self {
        log::error!("{}", value);
        Self::Io
    }
}

impl From<InfoError> for ElementError {
    fn from(value: InfoError) -> Self {
        log::error!("{}", value);
        Self::Info
    }
}
