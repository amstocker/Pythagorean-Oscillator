use crate::config;

use super::{Sample, Processor, EnvelopeDetector};


// Do we need to store state here?  Might be helpful to know
// when it's state has changed or not... (impl Processor for gate?)
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


pub struct CycleInfo {
    pub length: u32,
    pub start: bool
}

pub struct CycleDetector {
    high: Gate,
    low: Gate,
    cycle: Gate,
    high_env: EnvelopeDetector,
    low_env: EnvelopeDetector,
    sample_counter: u32,
    cycle_length: u32
}

impl CycleDetector {
    pub fn new() -> Self {
        CycleDetector {
            high: Gate::default(),
            low: Gate::default(),
            cycle: Gate::default(),
            high_env: EnvelopeDetector::new(
                config::CYCLE_DETECT_ENV_RISE,
                config::CYCLE_DETECT_ENV_FALL
            ),
            low_env: EnvelopeDetector::new(
                config::CYCLE_DETECT_ENV_FALL, 
                config::CYCLE_DETECT_ENV_RISE
            ),
            sample_counter: 0,
            cycle_length: 0
        }
    }
}

impl Processor<CycleInfo> for CycleDetector {
    fn process(&mut self, sample: Sample) -> CycleInfo {
        let high_env_value = self.high_env.process(sample.max(0.0));
        let low_env_value = self.low_env.process(sample.min(0.0));

        let mut high_off_to_on = false;
        if self.high.off() && sample > high_env_value {
            self.high.toggle();
            high_off_to_on = true;
        } else if self.high.on() && sample < high_env_value {
            self.high.toggle();
        }

        let mut low_off_to_on = false;
        if self.low.off() && sample < low_env_value {
            self.low.toggle();
            low_off_to_on = true;
        } else if self.low.on() && sample > low_env_value {
            self.low.toggle();
        }

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
        CycleInfo {
            length: self.cycle_length,
            start
        }
    }
}