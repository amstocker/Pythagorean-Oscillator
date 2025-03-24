use microfft::{Complex32, complex::cfft_2048};
use micromath::F32Ext;
use core::f32::consts::PI;

use crate::consts::*;
use crate::system::memory;
use crate::dsp::windowing;


pub struct Analyzer {
    windowing_func: &'static [f32],
    phase_history: &'static mut [f32]
}

impl Analyzer {
    pub fn init() -> Self {
        Analyzer {
            windowing_func:
                windowing::build_windowing_func(WINDOW_BUFFER_SIZE),
            phase_history:
                memory::allocate_f32_buffer(WINDOW_BUFFER_SIZE).unwrap()
        }
    }

    pub fn process(&mut self, window_buffer: &mut [Complex32]) -> f32 {
        for i in 0..WINDOW_BUFFER_SIZE {
            // Assume window_buffer[i].im == 0.0;
            window_buffer[i].re *= self.windowing_func[i];
        }

        let spectrum = cfft_2048(window_buffer.try_into().unwrap());

        let mut max_index = 0;
        let mut max_norm_sq = 0.0;
        let mut max_prev_phase = 0.0;
        for i in 0..(WINDOW_BUFFER_SIZE / 2) {
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
        let f_est = (max_index as f32 / WINDOW_BUFFER_SIZE as f32) * sample_rate; 
        let dt = 2.0 * PI * ((HOP_INTERVAL * BLOCK_LENGTH) as f32 / sample_rate);
        let dp = self.phase_history[max_index] - max_prev_phase;
        let mut p = 0.0;
        let mut f_prev = 0.0;
        loop {
            let f = (dp + p) / dt;
            if f > f_est {
                return if f - f_est < f_est - f_prev {
                    f
                } else {
                    f_prev
                };
            }
            f_prev = f;
            p += 2.0 * PI;
        }
    }
}