use super::Sample;


pub struct ZeroDetector {
    prev_sample: Sample
}

impl ZeroDetector {
    pub fn new() -> Self {
        ZeroDetector {
            prev_sample: Sample::default()
        }
    }
    
    pub fn process(&mut self, sample: Sample) -> bool {
        let mut zero_crossing = false;
        if !(sample * self.prev_sample > 0.0) {
            zero_crossing = true;
        }
        self.prev_sample = sample;
        zero_crossing
    }
}