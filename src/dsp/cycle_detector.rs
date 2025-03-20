use super::Sample;
use super::env_detector::EnvelopeDetector;
use super::gate::Gate;
use super::lpf::LowPassFilter;


pub struct Config {
    pub lpf_freq: f32,
    pub env_rise: f32,
    pub env_fall: f32
}

pub struct CycleDetectInfo {
    pub length: u32,
    pub start: bool
}

pub struct CycleDetector {
    high: Gate,
    low: Gate,
    cycle: Gate,
    lpf: LowPassFilter,
    high_env: EnvelopeDetector,
    low_env: EnvelopeDetector,
    sample_counter: u32,
    cycle_length: u32
}

impl CycleDetector {
    pub fn new(config: Config) -> Self {
        CycleDetector {
            high: Gate::default(),
            low: Gate::default(),
            cycle: Gate::default(),
            lpf: LowPassFilter::new(config.lpf_freq),
            high_env: EnvelopeDetector::new(
                config.env_rise,
                config.env_fall
            ),
            low_env: EnvelopeDetector::new(
                config.env_fall, 
                config.env_rise
            ),
            sample_counter: 0,
            cycle_length: 0
        }
    }
    
    pub fn process(&mut self, sample: Sample) -> CycleDetectInfo {
        let lpf_value = self.lpf.process(sample);
        let high_env_value = self.high_env.process(lpf_value.max(0.0));
        let low_env_value = self.low_env.process(lpf_value.min(0.0));

        let high_off_to_on = if self.high.off() && lpf_value > high_env_value {
            self.high.toggle();
            true
        } else if self.high.on() && lpf_value < high_env_value {
            self.high.toggle();
            false
        } else {
            false
        };

        let low_off_to_on = if self.low.off() && lpf_value < low_env_value {
            self.low.toggle();
            true
        } else if self.low.on() && lpf_value > low_env_value {
            self.low.toggle();
            false
        } else {
            false
        };

        let start = if self.cycle.off() && high_off_to_on {
            self.cycle.toggle();
            self.cycle_length = self.sample_counter;
            self.sample_counter = 0;
            true
        } else if self.cycle.on() && low_off_to_on {
            self.cycle.toggle();
            false
        } else {
            false
        };

        self.sample_counter += 1;

        CycleDetectInfo {
            length: self.cycle_length,
            start
        }
    }
}