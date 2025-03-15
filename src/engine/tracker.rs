use defmt::debug;

use crate::dsp::{CycleDetector, LowPassFilter, Processor, ZeroDetector};


// NOTE: this range includes the index
// at the end _after_ the start of a new cycle
#[derive(Default)]
pub struct Cycle {
    start: usize,
    end: usize
}

impl Cycle {
    pub fn is_degenerate(&self) -> bool {
        self.start == self.end
    }
}

pub struct CycleTracker<const N: usize> {
    buffer: [f32; N],
    index: usize,
    lpf: LowPassFilter,
    cycle_detector: CycleDetector,
    zero_detector: ZeroDetector,
    last_zero: usize,
    current_cycle: Cycle
}

impl<const N: usize> CycleTracker<N> {
    pub fn new() -> Self {
        CycleTracker {
            buffer: [0.0; N],
            index: 0,
            lpf: LowPassFilter::new(0.01),
            cycle_detector: CycleDetector::new(),
            zero_detector: ZeroDetector::new(),
            last_zero: 0,
            current_cycle: Cycle::default()
        }
    }
}

impl<const N: usize> Processor<()> for CycleTracker<N> {
    fn process(&mut self, sample: crate::dsp::Sample) -> () {
        self.buffer[self.index] = sample;
        let lpf_value = self.lpf.process(sample);
        
        let cycle_info = self.cycle_detector.process(lpf_value);
        if cycle_info.start {
            self.current_cycle = Cycle {
                start: self.current_cycle.end,
                end: self.last_zero
            };
            debug!("({}, {}) length = {}",
                self.current_cycle.start,
                self.current_cycle.end,
                (self.current_cycle.end + N - self.current_cycle.start) % N
            );
        }
        
        if self.zero_detector.process(sample) {
            self.last_zero = self.index;
        }

        self.index = (self.index + 1) % N;
    }
}