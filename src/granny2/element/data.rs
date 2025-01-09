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
    Reference(u64),
    Array(u64),
}
