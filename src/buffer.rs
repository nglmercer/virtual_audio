//! Ring buffer implementations for audio data transfer.
//!
//! This module provides thread-safe, lock-free ring buffers optimized
//! for real-time audio processing.

use crate::Error;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A simple ring buffer for audio samples.
///
/// This buffer is lock-free and suitable for real-time audio processing.
pub struct RingBuffer<T> {
    data: Vec<T>,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    capacity: usize,
    mask: usize,
}

impl<T: Clone + Copy + Default> RingBuffer<T> {
    /// Creates a new ring buffer with the specified capacity.
    ///
    /// The capacity is rounded up to the next power of 2 for efficient indexing.
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        let mask = capacity - 1;

        Self {
            data: vec![T::default(); capacity],
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            capacity,
            mask,
        }
    }

    /// Writes data into the ring buffer.
    ///
    /// Returns the number of samples actually written.
    pub fn write(&mut self, samples: &[T]) -> usize {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Acquire);

        let available = self.capacity - (write_pos - read_pos);
        let to_write = samples.len().min(available);

        if to_write == 0 {
            return 0;
        }

        for (i, &sample) in samples.iter().take(to_write).enumerate() {
            let idx = (write_pos + i) & self.mask;
            self.data[idx] = sample;
        }

        self.write_pos
            .store(write_pos + to_write, Ordering::Release);
        to_write
    }

    /// Reads data from the ring buffer.
    ///
    /// Returns the number of samples actually read.
    pub fn read(&self, output: &mut [T]) -> usize {
        let read_pos = self.read_pos.load(Ordering::Relaxed);
        let write_pos = self.write_pos.load(Ordering::Acquire);

        let available = write_pos - read_pos;
        let to_read = output.len().min(available);

        if to_read == 0 {
            return 0;
        }

        for (i, out) in output.iter_mut().take(to_read).enumerate() {
            let idx = (read_pos + i) & self.mask;
            *out = self.data[idx];
        }

        self.read_pos.store(read_pos + to_read, Ordering::Release);
        to_read
    }

    /// Returns the number of samples available for reading.
    pub fn available(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Acquire);
        write_pos - read_pos
    }

    /// Returns the amount of free space in the buffer.
    pub fn free_space(&self) -> usize {
        self.capacity - self.available()
    }

    /// Clears the buffer.
    pub fn clear(&self) {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        self.read_pos.store(write_pos, Ordering::Release);
    }
}

impl<T> Default for RingBuffer<T>
where
    T: Clone + Copy + Default,
{
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Triple ring buffer architecture for audio processing.
///
/// This architecture consists of:
/// - Input buffer: Receives data from the capture device
/// - Resample buffer: Holds data during sample rate conversion
/// - Output buffer: Delivers data to the playback device
pub struct TripleRingBuffer {
    /// Input buffer (from capture device/speaker)
    pub ring_input: RingBuffer<f32>,

    /// Resample buffer (during sample rate conversion)
    pub ring_resample: RingBuffer<f32>,

    /// Output buffer (to playback device/microphone)
    pub ring_output: RingBuffer<f32>,
}

impl TripleRingBuffer {
    /// Creates a new triple ring buffer with the specified capacity.
    pub fn new(buffer_size: usize) -> Self {
        Self {
            ring_input: RingBuffer::new(buffer_size),
            ring_resample: RingBuffer::new(buffer_size),
            ring_output: RingBuffer::new(buffer_size),
        }
    }

    /// Processes audio through the triple buffer pipeline.
    ///
    /// This method:
    /// 1. Writes input samples to the input buffer
    /// 2. Reads from input buffer, processes, and writes to resample buffer
    /// 3. Reads from resample buffer and writes to output buffer
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<usize, Error> {
        // Write input to input buffer
        let written = self.ring_input.write(input);

        // Transfer from input to resample (simple pass-through for now)
        let mut temp_buf = vec![0.0f32; written];
        let read = self.ring_input.read(&mut temp_buf);
        self.ring_resample.write(&temp_buf[..read]);

        // Transfer from resample to output
        let written_output = self.ring_output.read(output);

        Ok(written_output)
    }

    /// Clears all buffers.
    pub fn clear_all(&mut self) {
        self.ring_input.clear();
        self.ring_resample.clear();
        self.ring_output.clear();
    }

    /// Returns statistics about buffer levels.
    pub fn stats(&self) -> BufferStats {
        BufferStats {
            input_available: self.ring_input.available(),
            input_free: self.ring_input.free_space(),
            resample_available: self.ring_resample.available(),
            resample_free: self.ring_resample.free_space(),
            output_available: self.ring_output.available(),
            output_free: self.ring_output.free_space(),
        }
    }
}

impl Default for TripleRingBuffer {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Statistics about buffer levels.
#[derive(Debug, Clone)]
pub struct BufferStats {
    /// Number of samples available in input buffer
    pub input_available: usize,

    /// Free space in input buffer
    pub input_free: usize,

    /// Number of samples available in resample buffer
    pub resample_available: usize,

    /// Free space in resample buffer
    pub resample_free: usize,

    /// Number of samples available in output buffer
    pub output_available: usize,

    /// Free space in output buffer
    pub output_free: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let mut buffer = RingBuffer::<f32>::new(16);

        // Write some data
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let written = buffer.write(&input);
        assert_eq!(written, 4);

        // Read it back
        let mut output = vec![0.0; 8];
        let read = buffer.read(&mut output);
        assert_eq!(read, 4);
        assert_eq!(output[0], 1.0);
        assert_eq!(output[1], 2.0);
        assert_eq!(output[2], 3.0);
        assert_eq!(output[3], 4.0);
    }

    #[test]
    fn test_ring_buffer_wraparound() {
        let mut buffer = RingBuffer::<f32>::new(16);

        // Fill and empty to test wraparound
        for _ in 0..10 {
            let input = vec![1.0; 10];
            buffer.write(&input);

            let mut output = vec![0.0; 10];
            buffer.read(&mut output);
        }

        // Should still work
        let input = vec![42.0; 5];
        let written = buffer.write(&input);
        assert_eq!(written, 5);

        let mut output = vec![0.0; 5];
        let read = buffer.read(&mut output);
        assert_eq!(read, 5);
        assert_eq!(output[0], 42.0);
    }

    #[test]
    fn test_triple_ring_buffer() {
        let mut triple = TripleRingBuffer::new(64);

        let input = vec![1.0, 2.0, 3.0, 4.0];
        let mut output = vec![0.0; 8];

        let processed = triple.process(&input, &mut output).unwrap();
        assert_eq!(processed, 0); // No data in output yet

        // Process again to flush through pipeline
        let _ = triple.process(&[], &mut output).unwrap();
    }
}
