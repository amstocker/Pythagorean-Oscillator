use microfft::{Complex32, complex::cfft_2048};
use micromath::F32Ext;
use core::f32::consts::PI;

use crate::consts::*;
use crate::system::memory;
use crate::dsp::windowing;


pub struct Analyzer {
    windowing_func: &'static [f32],
    phase_history: &'static mut [f32],
    frequency: f32
}

impl Analyzer {
    pub fn init() -> Self {
        Analyzer {
            windowing_func: windowing::build_windowing_func(BUFFER_SIZE),
            phase_history: memory::allocate_buffer(BUFFER_SIZE).unwrap(),
            frequency: 0.0
        }
    }

    pub fn process(&mut self, window_buffer: &mut [f32]) {
        for i in 0..BUFFER_SIZE {
            window_buffer[2 * i] *= self.windowing_func[i];
        }

        let samples = {
            let slice = unsafe {
                let ptr = window_buffer.as_mut_ptr().cast::<Complex32>();
                core::slice::from_raw_parts_mut(ptr, BUFFER_SIZE)
            };
            slice.try_into().unwrap()
        };

        let spectrum = cfft_2048(samples);

        let mut max_index = 0;
        let mut max_norm_sq = 0.0;
        let mut max_prev_phase = 0.0;
        for i in 0..(BUFFER_SIZE / 2) {
            let Complex32 { re, im} = spectrum[i];
            let norm_sq = re * re + im * im;
            if norm_sq > max_norm_sq {
                max_norm_sq = norm_sq;
                max_index = i;
                max_prev_phase = self.phase_history[i];
            }
            self.phase_history[i] = im.atan2(re);
        }

        let sample_rate = SAMPLE_RATE as f32;
        let f_est = (max_index as f32 / BUFFER_SIZE as f32) * sample_rate; 
        let dt = 2.0 * PI * ((HOP_INTERVAL * BLOCK_LENGTH) as f32 / sample_rate);
        let dp = self.phase_history[max_index] - max_prev_phase;
        let mut p = 0.0;
        let mut f_prev = 0.0;
        loop {
            let f = (dp + p) / dt;
            if f > f_est {
                self.frequency = if f - f_est < f_est - f_prev {
                    f
                } else {
                    f_prev
                };
                break;
            }
            f_prev = f;
            p += 2.0 * PI;
        }
    }

    pub fn frequency(&self) -> f32 {
        self.frequency
    }
}