use std::io::Read;

#[derive(Debug)]
pub struct MarshallingHeader {
    pub offset: u32,
    pub count: u32,
}

impl MarshallingHeader {
    pub fn parse<T: Read>(reader: &mut T) -> Result<Self, std::io::Error> {
        log::trace!("Parsing marshalling header");
        let offset = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let count = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let marshalling_header = Self { offset, count };

        Ok(marshalling_header)
    }
}
