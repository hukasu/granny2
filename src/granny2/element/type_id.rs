#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeId {
    /// No node
    None,
    /// Empty node with just children
    Inline,
    /// Reference to a pointer
    Reference,
    /// Reference to an array
    ReferenceToArray,
    /// Array containing a numbers of pointers
    ArrayOfReferences,
    /// Reference with offset
    VariantReference,
    /// TODO: We know this was used to be reference or a custom type, is there anything that reference this?
    Removed,
    /// Reference to an array with offset
    ReferenceToVariantArray,
    /// String
    String,
    /// Transform
    Transform,
    /// 32bit floating value
    Real32,
    /// 8bit number signed
    Int8,
    /// 8bit number unsigned
    UInt8,
    /// TODO: discover what changes between this and int8
    Int8Norm,
    /// TODO: discover what changes between this and uint8
    UInt8Norm,
    /// 16bit number signed
    Int16,
    /// 16bit number unsigned
    UInt16,
    /// TODO: discover what changes between this and int16
    Int16Norm,
    /// TODO: discover what changes between this and uint16
    UInt16Norm,
    /// 32bit number signed
    Int32,
    /// 32bit number unsigned
    UInt32,
    /// half-sized floating value
    Real16,
    /// Reference to nothing
    EmptyReference,
}

impl TryFrom<u32> for TypeId {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Inline),
            2 => Ok(Self::Reference),
            3 => Ok(Self::ReferenceToArray),
            4 => Ok(Self::ArrayOfReferences),
            5 => Ok(Self::VariantReference),
            6 => Ok(Self::Removed),
            7 => Ok(Self::ReferenceToVariantArray),
            8 => Ok(Self::String),
            9 => Ok(Self::Transform),
            10 => Ok(Self::Real32),
            11 => Ok(Self::Int8),
            12 => Ok(Self::UInt8),
            13 => Ok(Self::Int8Norm),
            14 => Ok(Self::UInt8Norm),
            15 => Ok(Self::Int16),
            16 => Ok(Self::UInt16),
            17 => Ok(Self::Int16Norm),
            18 => Ok(Self::UInt16Norm),
            19 => Ok(Self::Int32),
            20 => Ok(Self::UInt32),
            21 => Ok(Self::Real16),
            22 => Ok(Self::EmptyReference),
            other => Err(other),
        }
    }
}
