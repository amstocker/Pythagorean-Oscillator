use super::Sample;


pub struct Config {
    pub rise: f32,
    pub fall: f32
}

pub struct EnvelopeDetector {
    rise: f32,
    fall: f32,
    value: Sample
}

impl EnvelopeDetector {
    pub fn new(rise: f32, fall: f32) -> Self {
        EnvelopeDetector {
            rise,
            fall,
            value: Sample::default()
        }
    }
    
    pub fn process(&mut self, sample: Sample) -> Sample {
        let rate = if sample > self.value {
            self.rise
        } else {
            self.fall
        };
        self.value += rate * (sample - self.value);
        self.value
    }
}