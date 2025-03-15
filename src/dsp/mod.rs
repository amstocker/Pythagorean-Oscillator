pub mod cycle;
pub mod zero;

pub use cycle::*;
pub use zero::*;


pub type Sample = f32;

pub trait Processor<T> {
    fn process(&mut self, sample: Sample) -> T;
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
}

impl Processor<Sample> for EnvelopeDetector {
    fn process(&mut self, sample: Sample) -> Sample {
        let rate = if sample > self.value {
            self.rise
        } else {
            self.fall
        };
        self.value += rate * (sample - self.value);
        self.value
    }
}

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
}

impl Processor<Sample> for LowPassFilter {
    fn process(&mut self, sample: Sample) -> Sample {
        self.value += self.decay * (0.5 * (sample + self.prev_sample) - self.value);
        self.prev_sample = sample;
        self.value
    }
}