pub mod tracker;
pub mod voice;

use crate::config::ENGINE_CONFIG;
use crate::{dsp::Sample, memory};
pub use tracker::CycleTracker;
pub use voice::Voice;

use core::f32::consts::PI;
use micromath::F32Ext;


fn build_sine_buffer(len: usize) -> &'static mut [f32] {
    let buffer = memory::allocate_buffer(len).unwrap();
    for i in 0..len {
        buffer[i] = (2.0 * PI * (i as f32 / len as f32)).sin();
    }
    buffer
}

pub struct Config {
    pub num_voices: usize,
    pub voice_config: voice::Config,
    pub tracker_config: tracker::Config
}

pub struct Engine {
    cycle_tracker: CycleTracker,
    voices: [Voice; ENGINE_CONFIG.num_voices],
    num_voices: usize,
    voice_amp: f32,
    voice_index: usize
}

impl Engine {
    pub fn new() -> Self {
        let buffer = build_sine_buffer(512);
        let voices = [(); ENGINE_CONFIG.num_voices].map(
            |_| {
                let mut voice = Voice::new(ENGINE_CONFIG.voice_config);
                voice.set_waveform(buffer, 512);
                voice
            }
        );
        defmt::debug!("Engine init (memory capacity: {}/{})", memory::capacity(), memory::size());
        Engine {
            cycle_tracker: CycleTracker::new(ENGINE_CONFIG.tracker_config),
            voices,
            num_voices: ENGINE_CONFIG.num_voices,
            voice_amp: 1.0 / ENGINE_CONFIG.num_voices as f32,
            voice_index: 0,
        }
    }
    
    pub fn process(&mut self, sample: Sample) -> Sample {
        let cycle = self.cycle_tracker.process(sample);

        if cycle.fresh && cycle.length > 0 {
            self.voices[self.voice_index].set_rate(cycle.length);
            self.voice_index = (self.voice_index + 1) % self.num_voices;
        }

        let mut sample = 0.0;
        for voice in self.voices.iter_mut() {
            sample += self.voice_amp * voice.next_sample();
        }
        sample
    }
}