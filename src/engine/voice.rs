use micromath::F32Ext;

use crate::{dsp::{Processor, Sample}, memory};


pub struct Voice<const N: usize> {
    buffer: &'static mut [f32],
    swap_buffer: &'static mut [f32],
    buffer_length: usize,
    samples_per_cycle: usize,
    phase: f32,
    phase_incr: f32,
    queue_buffer_swap: bool,
    queue_buffer_length: usize,
    fixed_rate: bool
}

impl<const N: usize> Voice<N> {
    pub fn new(fixed_rate: bool) -> Self {
        Voice {
            buffer: memory::allocate_buffer(N).unwrap(),
            swap_buffer: memory::allocate_buffer(N).unwrap(),
            buffer_length: 0,
            samples_per_cycle: 0,
            phase: 0.0,
            phase_incr: 0.0,
            queue_buffer_swap: false,
            queue_buffer_length: 0,
            fixed_rate
        }
    }

    fn update_phase_incr(&mut self) {
        self.phase_incr = 1.0 / self.samples_per_cycle as f32;
    }

    pub fn set_rate(&mut self, samples_per_cycle: usize) {
        self.samples_per_cycle = samples_per_cycle;
        self.update_phase_incr();
    }

    pub fn set_waveform(&mut self, buffer: &[f32], length: usize) {
        self.queue_buffer_swap = true;
        self.queue_buffer_length = length;
        for i in 0..length {
            self.swap_buffer[i] = buffer[i];
        }
    }
}

impl<const N: usize> Processor for Voice<N> {
    fn process(&mut self, _sample: Sample) -> Sample {

        // TODO: Create more sophisticated interpolation methods
        //       in dsp module.
        let sample = {
            let index_approx = self.phase * self.buffer_length as f32;
            let index_floor = index_approx.floor();
            let t = index_approx - index_floor;
            let i = index_floor as usize;
            
            (1.0 - t) * self.buffer[i] + t * self.buffer[(i + 1) % N]
        };

        self.phase += self.phase_incr;
        if self.phase > 1.0 {
            self.phase -= 1.0;
            if self.queue_buffer_swap {
                self.queue_buffer_swap = false;
                self.buffer_length = self.queue_buffer_length;
                if !self.fixed_rate {
                    self.set_rate(self.buffer_length);
                }
                self.update_phase_incr();

                // TODO: Double check this actually works!
                core::mem::swap(&mut self.buffer, &mut self.swap_buffer);
            }
        }

        sample
    }
}