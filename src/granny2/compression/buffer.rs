pub struct Buffer {
    buffer: Vec<u8>,
    position: usize,
}

impl Buffer {
    pub fn new(length: usize) -> Self {
        Self {
            buffer: vec![0; length],
            position: 0,
        }
    }

    pub fn inner(self) -> Vec<u8> {
        self.buffer
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn advance(&mut self, shift: usize) {
        self.position += shift;
    }

    pub fn push(&mut self, value: u8) {
        self.buffer[self.position] = value;
    }

    pub fn backref(&mut self, backref_size: usize, backref_offset: usize) {
        let (back, mut front) = self.buffer.split_at_mut(self.position);
        let back = &back[(back.len() - backref_offset)..];

        let repeat = backref_size / backref_offset;
        let remain = backref_size % backref_offset;

        for _ in 0..repeat {
            front.copy_from_slice(back);
            front = &mut front[backref_offset..];
        }
        front[..remain].copy_from_slice(&back[..remain]);
    }
}
