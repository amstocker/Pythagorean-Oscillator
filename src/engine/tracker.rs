use crate::memory;
use crate::dsp::{CycleDetector, LowPassFilter, Processor, ZeroDetector};


// keep track of last M cycles, but need to make sure they are popped after
// being overwritten?
const CYCLE_HISTORY: usize = 16;


// NOTE: this range includes the index
// at the end _after_ the start of a new cycle
#[derive(Default, Clone, Copy)]
pub struct Cycle<const N: usize> {
    start: usize,
    end: usize
}

impl<const N: usize> Cycle<N> {
    pub fn length(&self) -> usize {
        (self.end + N - self.start) % N
    }

    pub fn is_degenerate(&self) -> bool {
        self.start == self.end
    }
}

pub struct CycleTracker<const N: usize> {
    buffer: &'static mut [f32],
    index: usize,
    lpf: LowPassFilter,
    cycle_detector: CycleDetector,
    zero_detector: ZeroDetector,
    last_zero: usize,
    last_cycle: Cycle<N>
    // count frames since end of last cycle?
}

impl<const N: usize> CycleTracker<N> {
    pub fn new() -> Self {
        CycleTracker {
            buffer: memory::allocate_buffer(N).unwrap(),
            index: 0,
            lpf: LowPassFilter::new(0.01),
            cycle_detector: CycleDetector::new(),
            zero_detector: ZeroDetector::new(),
            last_zero: 0,
            last_cycle: Cycle::default()
        }
    }
}

impl<const N: usize> Processor<Cycle<N>> for CycleTracker<N> {
    fn process(&mut self, sample: crate::dsp::Sample) -> Cycle<N> {
        self.buffer[self.index] = sample;
        let lpf_value = self.lpf.process(sample);
        
        let cycle_info = self.cycle_detector.process(lpf_value);
        if cycle_info.start {
            self.last_cycle = Cycle {
                start: self.last_cycle.end,
                end: self.last_zero
            };
        }
        
        if self.zero_detector.process(sample) {
            self.last_zero = self.index;
        }

        self.index = (self.index + 1) % N;
        self.last_cycle
    }
}