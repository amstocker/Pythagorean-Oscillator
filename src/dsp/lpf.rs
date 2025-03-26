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
    value: f32,
    prev_sample: f32,
    pub dc_gain: f32
}

impl LowPassFilter {
    pub fn new(freq: f32) -> Self {
        let decay = hz_to_lpf_decay(freq);
        LowPassFilter {
            decay,
            value: 0.0,
            prev_sample: 0.0,
            dc_gain: (1.0 + 0.12) / decay
        }
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.decay = hz_to_lpf_decay(freq);
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.value += (sample + 0.12 * self.prev_sample) - self.decay * self.value;
        self.prev_sample = sample;
        self.value
    }
}
