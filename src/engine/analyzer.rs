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
                // Only store information about frequencies under Nyquist.
                memory::allocate_f32_buffer(WINDOW_BUFFER_SIZE / 2).unwrap()
        }
    }

    pub fn process(&mut self, window_buffer: &mut [Complex32]) -> f32 {
        // We analyze the frequency using a phase vocoder.
        //   (See: https://sethares.engr.wisc.edu/vocoders/phasevocoder.html)
        // 
        // Steps:
        //  1. Multiply the window buffer with a windowing function.
        //  2. Calculate the discrete fourier transform of the window.
        //  3. Find the frequency bucket with peak magnitude.
        //  4. Use the difference in phase at the peak magnitude to get
        //     an accurate estimate for the frequency of the incoming
        //     signal.

        // Step 1
        for i in 0..WINDOW_BUFFER_SIZE {
            // Assume window_buffer[i].im == 0.0;
            window_buffer[i].re *= self.windowing_func[i];
        }

        // Step 2
        let spectrum = cfft_2048(window_buffer.try_into().unwrap());

        // Step 3
        let mut max_index = 0;
        let mut max_norm_sq = 0.0;
        let mut max_prev_phase = 0.0;
        for i in 0..(WINDOW_BUFFER_SIZE / 2) {
            let Complex32 { re, im } = spectrum[i];
            let norm_sq = re * re + im * im;
            if norm_sq > max_norm_sq {
                max_norm_sq = norm_sq;
                max_index = i;
                max_prev_phase = self.phase_history[i];
            }
            self.phase_history[i] = im.atan2(re);
        }

        // Step 4:
        // - Our initial estimate for the frequency is the center frequency of the
        //   bucket with the maximum magnitude calculated above.
        // - To get a more accurate estimate for the frequency, we use the difference
        //   in phase.  This can only give us the frequency up to a multiple of 2pi,
        //   so we find the multiple that is closest to the estimated frequency.
        let sample_rate = SAMPLE_RATE as f32;
        let freq_est = ((max_index as f32 + 0.5) / WINDOW_BUFFER_SIZE as f32) * sample_rate; 
        let dt = 2.0 * PI * ((HOP_INTERVAL * BLOCK_LENGTH) as f32 / sample_rate);
        let dp = self.phase_history[max_index] - max_prev_phase;
        let mut phase = 0.0;
        let mut freq_prev = 0.0;
        loop {
            let freq = (dp + phase) / dt;
            if freq > freq_est {
                return if freq - freq_est < freq_est - freq_prev {
                    freq
                } else {
                    freq_prev
                };
            }
            freq_prev = freq;
            phase += 2.0 * PI;
        }
    }
}