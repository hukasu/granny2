use std::io::Read;

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    flags: u32,
    translation: [f32; 3],
    rotation: [f32; 4],
    scale_shear: [f32; 9],
}

impl Transform {
    pub fn parse<T: Read>(reader: &mut T) -> Result<Self, std::io::Error> {
        let flags = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            u32::from_le_bytes(buffer)
        };
        let translation = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            let x = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let y = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let z = f32::from_le_bytes(buffer);
            [x, y, z]
        };
        let rotation = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            let i = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let j = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let k = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let w = f32::from_le_bytes(buffer);
            [i, j, k, w]
        };
        let scale_shear = {
            let mut buffer = [0; 4];
            reader.read_exact(&mut buffer)?;
            let x1 = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let y1 = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let z1 = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let x2 = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let y2 = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let z2 = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let x3 = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let y3 = f32::from_le_bytes(buffer);
            reader.read_exact(&mut buffer)?;
            let z3 = f32::from_le_bytes(buffer);
            [x1, y1, z1, x2, y2, z2, x3, y3, z3]
        };

        Ok(Self {
            flags,
            translation,
            rotation,
            scale_shear,
        })
    }
}
