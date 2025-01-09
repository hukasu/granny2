use std::io::Read;

#[derive(Debug)]
pub struct Relocation {
    pub src_offset: usize,
    pub dst_section: usize,
    pub dst_offset: usize,
}

impl Relocation {
    pub fn parse<T: Read>(reader: &mut T) -> Result<Self, std::io::Error> {
        let src_offset = {
            let mut buffer = [0; 8];
            reader.read_exact(&mut buffer[0..4])?;
            usize::from_le_bytes(buffer)
        };
        let dst_section = {
            let mut buffer = [0; 8];
            reader.read_exact(&mut buffer[0..4])?;
            usize::from_le_bytes(buffer)
        };
        let dst_offset = {
            let mut buffer = [0; 8];
            reader.read_exact(&mut buffer[0..4])?;
            usize::from_le_bytes(buffer)
        };

        Ok(Self {
            src_offset,
            dst_section,
            dst_offset,
        })
    }

    pub const fn sizeof() -> usize {
        12
    }
}
