use crate::granny2::transform::Transform;

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Empty,
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Real32(f32),
    Transform(Transform),
    String(Box<str>),
    Array(u64, u64),
    Reference(u64),
    ArrayOfReferences(Vec<u64>),
    Variant(u64, u64),
    VariantArray(u64, u64, u64),
}
