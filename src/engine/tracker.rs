use crate::memory;
use crate::dsp::{
    Sample,
    cycle_detector::{self, CycleDetector},
    zero::ZeroDetector
};


pub struct Config {
    pub buffer_size: usize,
    pub cycle_detector_config: cycle_detector::Config
}

#[derive(Default, Clone, Copy)]
pub struct Cycle {
    pub start: usize,
    pub end: usize,
    pub length: usize,
    pub fresh: bool
}

pub struct CycleTracker {
    buffer: &'static mut [f32],
    buffer_size: usize,
    index: usize,
    cycle_detector: CycleDetector,
    zero_detector: ZeroDetector,
    last_zero: usize,
    last_cycle: Cycle
}

impl CycleTracker {
    pub fn new(config: Config) -> Self {
        CycleTracker {
            buffer: memory::allocate_buffer(config.buffer_size).unwrap(),
            buffer_size: config.buffer_size,
            index: 0,
            cycle_detector: CycleDetector::new(config.cycle_detector_config),
            zero_detector: ZeroDetector::new(),
            last_zero: 0,
            last_cycle: Cycle::default()
        }
    }
    
    pub fn process(&mut self, sample: Sample) -> Cycle {
        self.buffer[self.index] = sample;
        let cycle_info = self.cycle_detector.process(sample);
        if cycle_info.start {
            self.last_cycle = Cycle {
                start: self.last_cycle.end,
                end: self.last_zero,
                length: (self.last_zero + self.buffer_size - self.last_cycle.end) % self.buffer_size,
                fresh: true
            };
        } else {
            self.last_cycle.fresh = false;
        }
        
        if self.zero_detector.process(sample) {
            self.last_zero = self.index;
        }

        self.index = (self.index + 1) % self.buffer_size;
        self.last_cycle
    }
}