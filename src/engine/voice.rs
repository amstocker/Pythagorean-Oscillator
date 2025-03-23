use micromath::F32Ext;

use crate::dsp::Sample;
use crate::system::memory;


#[derive(Clone, Copy)]
pub struct Config {
    pub buffer_size: usize
}

pub struct Voice {
    pub buffer: &'static mut [f32],
    pub buffer_size: usize,
    swap_buffer: &'static mut [f32],
    buffer_length: usize,
    samples_per_cycle: usize,
    phase: f32,
    phase_incr: f32,
    queue_buffer_swap: bool,
    queue_buffer_length: usize
}

impl Voice {
    pub fn new(config: Config) -> Self {
        Voice {
            buffer: memory::allocate_buffer(config.buffer_size).unwrap(),
            buffer_size: config.buffer_size,
            swap_buffer: memory::allocate_buffer(config.buffer_size).unwrap(),
            buffer_length: 0,
            samples_per_cycle: 0,
            phase: 0.0,
            phase_incr: 1.0,
            queue_buffer_swap: false,
            queue_buffer_length: 0,
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
        if length > self.buffer_size {
            return;
        }
        self.queue_buffer_swap = true;
        self.queue_buffer_length = length;
        for i in 0..length {
            self.swap_buffer[i] = buffer[i];
        }
    }

    pub fn next_sample(&mut self) -> Sample {
        
        // TODO: Create more sophisticated interpolation methods
        //       in dsp module.
        let sample = {
            let index_approx = self.phase * self.buffer_length as f32;
            let index_floor = index_approx.floor();
            let t = index_approx - index_floor;
            let i = index_floor as usize;
            
            (1.0 - t) * self.buffer[i] + t * self.buffer[(i + 1) % self.buffer_size]
        };

        self.phase += self.phase_incr;
        if self.phase > 1.0 {
            self.phase -= 1.0;
            if self.queue_buffer_swap {
                self.queue_buffer_swap = false;
                self.buffer_length = self.queue_buffer_length;
                self.set_rate(self.buffer_length);
                self.update_phase_incr();

                core::mem::swap(&mut self.buffer, &mut self.swap_buffer);
            }
        }

        sample
    }
}
