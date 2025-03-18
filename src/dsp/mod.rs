pub mod cycle_detector;
pub mod zero;

pub use cycle_detector::*;
pub use zero::*;


pub type Sample = f32;

pub enum Rate {
    VeryFast,
    Fast,
    Medium,
    Slow,
    VerySlow
}

pub struct EnvelopeDetectorConfig {
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


#[derive(Default)]
pub enum Gate {
    On,
    #[default]
    Off
}

impl Gate {
    pub fn on(&self) -> bool {
        match self {
            Gate::On  => true,
            Gate::Off => false,
        }
    }

    pub fn off(&self) -> bool {
        match self {
            Gate::On  => false,
            Gate::Off => true,
        }
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Gate::On  => Gate::Off,
            Gate::Off => Gate::On,
        }
    }
}

impl From<Gate> for Sample {
    fn from(value: Gate) -> Self {
        match value {
            Gate::On  => 1.0,
            Gate::Off => 0.0,
        }
    }
}