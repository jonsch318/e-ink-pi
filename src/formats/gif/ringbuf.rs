use std::ops::{Index, IndexMut};

#[derive(Debug, Copy, Clone)]
pub struct RingBuffer<T, const CAP: usize> {
    data: [T; CAP],
    index: usize,
}

impl<T, const CAP: usize> RingBuffer<T, CAP>
where
    T: Default,
{
    pub fn new() -> Self {
        Self {
            data: std::array::from_fn(|_| T::default()),
            index: 0,
        }
    }
    pub fn new_from(data: [T; CAP], index: usize) -> Self {
        assert!(CAP > 0);
        Self { data, index }
    }

    pub fn current(&self) -> &T {
        &self.data[self.index]
    }
    pub fn last(&self) -> &T {
        &self.data[Self::wrapping_sub(self.index, 1)]
    }
    pub fn current_mut(&mut self) -> &mut T {
        &mut self.data[self.index]
    }
    pub fn last_mut(&mut self) -> &mut T {
        &mut self.data[Self::wrapping_sub(self.index, 1)]
    }

    pub fn next(&mut self) {
        self.index = Self::wrapping_add(self.index, 1);
    }

    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    fn wrapping_add(a: usize, b: usize) -> usize {
        (a + b) % CAP
    }
    #[inline]
    fn wrapping_sub(a: usize, b: usize) -> usize {
        (CAP + a - b) % CAP
    }
}

impl<T, const CAP: usize> Index<usize> for RingBuffer<T, CAP> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index % CAP]
    }
}

impl<T, const CAP: usize> IndexMut<usize> for RingBuffer<T, CAP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index % CAP]
    }
}

impl<T: Default + Copy, const CAP: usize> Default for RingBuffer<T, CAP>
where
    T: Default,
{
    fn default() -> Self {
        RingBuffer {
            index: 0,
            data: [T::default(); CAP],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer() {
        let mut buffer: RingBuffer<i32, 2> = RingBuffer::default();

        *buffer.current_mut() = 1;
        buffer.next();
        *buffer.current_mut() = 2;
        buffer.next();
        *buffer.current_mut() = 3;

        assert_eq!(*buffer.current(), 3);
        assert_eq!(*buffer.last(), 2);

        buffer.next();
        *buffer.current_mut() = 4;

        assert_eq!(*buffer.current(), 4);
        assert_eq!(*buffer.last(), 3);
    }
}
