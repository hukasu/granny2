use std::io::Read;

#[derive(Debug)]
pub struct Parameters {
    pub decoded_value_max: u32,
    pub backref_value_max: u32,
    pub decoded_count: u32,
    pub _padding: u32,
    pub highbit_count: u32,
    pub sizes_count: [u8; 4],
}

impl Parameters {
    pub fn parse<T: Read>(reader: &mut T) -> Result<Self, std::io::Error> {
        let top = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };
        let bottom = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };

        let decoded_value_max = (top & 0xff800000) >> 23;
        let backref_value_max = top & 0x007fffff;

        let decoded_count = (bottom & 0xff800000) >> 23;
        let _padding = (bottom & 0x007fe000) >> 13;
        let highbit_count = bottom & 0x00001fff;

        let sizes_count = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            buffer
        };

        Ok(Self {
            decoded_value_max,
            backref_value_max,
            decoded_count,
            _padding,
            highbit_count,
            sizes_count,
        })
    }
}
