use super::Sample;


pub struct LowPassFilter {
    decay: f32,
    value: Sample,
    prev_sample: Sample
}

impl LowPassFilter {
    pub fn new(decay: f32) -> Self {
        LowPassFilter {
            decay,
            value: Sample::default(),
            prev_sample: Sample::default()
        }
    }
    
    pub fn process(&mut self, sample: Sample) -> Sample {
        self.value += self.decay * (0.5 * (sample + self.prev_sample) - self.value);
        self.prev_sample = sample;
        self.value
    }
}
