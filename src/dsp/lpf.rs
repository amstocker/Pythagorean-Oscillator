use super::Sample;

use micromath::F32Ext;


pub fn hz_to_lpf_decay(freq: f32) -> f32 {
    use core::f32::consts::PI;
    use daisy::audio::FS;

    let f = freq / FS.to_Hz() as f32;
    let a = (-2.0 * PI * f).exp();
    1.0 - a
}


// Simple LPF (See: https://freeverb3-vst.sourceforge.io/doc/AN11.pdf):
//
//      y[n] = 1.0 * x[n] + 0.12 * x[n-1] + a * y[n-1]
//
// where a = exp(-2*PI*(f/f_S)).
pub struct LowPassFilter {
    decay: f32,
    value: Sample,
    prev_sample: Sample
}

impl LowPassFilter {
    pub fn new(freq: f32) -> Self {
        LowPassFilter {
            decay: hz_to_lpf_decay(freq),
            value: Sample::default(),
            prev_sample: Sample::default()
        }
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.decay = hz_to_lpf_decay(freq);
    }
    
    pub fn process(&mut self, sample: Sample) -> Sample {
        self.value += (sample + 0.12 * self.prev_sample) - self.decay * self.value;
        self.prev_sample = sample;
        self.value
    }
}
