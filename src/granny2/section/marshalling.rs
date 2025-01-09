#[derive(Debug)]
pub struct Marshalling {
    count: usize,
    src_offset: usize,
    dst_section: usize,
    dst_offset: usize,
}
