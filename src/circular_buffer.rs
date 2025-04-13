
pub struct CircularBuffer {
    buffer: Vec<f32>,
    pos: usize,
    buffer_size: usize,
}

impl CircularBuffer {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            buffer: vec![0.; buffer_size],
            pos: 0,
            buffer_size,
        }
    }

    pub fn append(&mut self, e: f32) {
        self.buffer[self.pos] = e;
        self.pos += 1;
        self.pos %= self.buffer_size;
    }

    pub fn append_slice(&mut self, e: &[f32]) {
        debug_assert!(e.len() <= self.buffer_size);

        if (self.buffer_size - self.pos) >= e.len() {
            self.buffer[self.pos..self.pos+e.len()].copy_from_slice(e);
            self.pos += e.len();
            self.pos %= self.buffer_size;
            return;
        }
        
        let split_idx = self.buffer_size - self.pos - 1;
        let (e_left, e_right) = e.split_at(split_idx + 1);

        self.buffer[self.pos..].copy_from_slice(e_left);
        self.buffer[0..(e.len() - split_idx - 1)].copy_from_slice(e_right);

        self.pos += e.len();
        self.pos %= self.buffer_size;
    }

    pub fn get_ordered(&self) -> Vec<f32> {
        if self.pos == 0 {
            return self.buffer.clone();
        }

        let mut out: Vec<f32> = vec![0.; self.buffer_size];

        let (left, right) = out.split_at_mut(self.buffer_size - self.pos);
        left.copy_from_slice(&self.buffer[self.pos..]);
        right.copy_from_slice(&self.buffer[0..self.pos]);
        
        out
    }

    pub fn get_ordered_slices(&self) -> (&[f32], &[f32]) {
        (&self.buffer[self.pos..], &self.buffer[0..self.pos])
    }
}
