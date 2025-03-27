use microfft::{Complex32, complex::cfft_1024};
use micromath::F32Ext;
use core::f32::consts::PI;

use crate::consts::*;
use crate::dsp::window::build_window;


pub struct Analyzer {
    buffer: [Complex32; WINDOW_BUFFER_SIZE],
    window: [f32; WINDOW_BUFFER_SIZE],
    phase_history: [f32; WINDOW_BUFFER_SIZE / 2]
}

impl Analyzer {
    pub fn init() -> Self {
        let mut analyzer = Analyzer {
            buffer:
                [Complex32::default(); WINDOW_BUFFER_SIZE],
            window:
                [0.0; WINDOW_BUFFER_SIZE],
            phase_history:
                // Only store information about frequencies under Nyquist.
                [0.0; WINDOW_BUFFER_SIZE / 2]
        };
        build_window(&mut analyzer.window);
        analyzer
    }

    pub fn process(&mut self, input_buffer: &mut [f32]) -> f32 {
        // We analyze the frequency using a phase vocoder.
        //   (See: https://sethares.engr.wisc.edu/vocoders/phasevocoder.html)
        // 
        // Steps:
        //  1. Multiply the window buffer with a windowing function.
        //  2. Calculate the discrete fourier transform of the window.
        //  3. Estimate frequency using magnitude spectrum.
        //  4. Use the difference in phase at the peak magnitude to get
        //     a much more accurate estimate for the frequency.

        // Step 1: Copy window to internal buffer with windowing function.
        for i in 0..WINDOW_BUFFER_SIZE {
            self.buffer[i] = Complex32 {
                re: input_buffer[i] * self.window[i],
                im: 0.0
            };
        }

        // Step 2
        let spectrum = cfft_1024(&mut self.buffer);

        // Step 3 (TODO: parabolic interpolation of max)
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
        let dt = 2.0 * PI * (WINDOW_HOP as f32 / sample_rate);
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