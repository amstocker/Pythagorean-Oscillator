use micromath::F32Ext;

use crate::consts::DSP_BUFFER_SIZE;
use crate::system::memory;


pub struct Prism {
    buffer: &'static mut [f32],
    pub index: usize,

    // `frequency`` has units of samples per cycle.
    frequency: f32,
    pub phase_sub_oct: f32,
    pub phase_fifth: f32,
    feedback: f32,
    balance: f32
}

impl Prism {
    pub fn init() -> Self {
        Prism {
            buffer: memory::allocate_f32_buffer(DSP_BUFFER_SIZE).unwrap(),
            index: 0,
            frequency: 0.0,
            phase_sub_oct: 0.0,
            phase_fifth: 0.0,
            feedback: 0.7,
            balance: 0.7
        }
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        let mut processed_sample = (1.0 - self.feedback) * sample;

        // Sub-Oct
        let left = self.phase_sub_oct.floor();
        let right = (left as usize + 1) % DSP_BUFFER_SIZE;
        let t = self.phase_sub_oct - left;

        let x = (1.0 - t) * self.buffer[left as usize] + t * self.buffer[right];
        processed_sample += self.feedback * (1.0 - self.balance) * x;

        self.phase_sub_oct += 0.5;
        if !(self.phase_sub_oct < DSP_BUFFER_SIZE as f32) {
            self.phase_sub_oct -= DSP_BUFFER_SIZE as f32;
        }

        // Fifth
        let left = self.phase_fifth.floor();
        let right = (left as usize + 1) % DSP_BUFFER_SIZE;
        let t = self.phase_fifth - left;

        let x = (1.0 - t) * self.buffer[left as usize] + t * self.buffer[right];
        processed_sample += self.feedback * self.balance * x;

        self.phase_fifth += 1.5;
        if !(self.phase_fifth < DSP_BUFFER_SIZE as f32) {
            self.phase_fifth -= DSP_BUFFER_SIZE as f32;
        }


        self.buffer[self.index] = processed_sample;
        self.index = (self.index + 1) % DSP_BUFFER_SIZE;
        processed_sample
    }

    pub fn set_fundamental_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}