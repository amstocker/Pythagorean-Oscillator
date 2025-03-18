mod tracker;
mod voice;

use crate::{dsp::Sample, memory};
pub use tracker::{CycleTracker, Config as CycleTrackerConfig};
pub use voice::{Voice, Config as VoiceConfig};

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
    pub voice_config: VoiceConfig,
    pub cycle_tracker_config: CycleTrackerConfig
}

pub struct Engine<const N: usize> {
    cycle_tracker: CycleTracker,
    voices: [Voice; N],
    voice_amp: f32,
    voice_index: usize
}

impl<const N: usize> Engine<N> {
    pub fn new(config: Config) -> Self {
        let buffer = build_sine_buffer(512);
        let voices = [(); N].map(
            |_| {
                let mut voice = Voice::new(config.voice_config);
                voice.set_waveform(buffer, 512);
                voice
            }
        );
        defmt::debug!("Engine init (memory capacity: {}/{})", memory::capacity(), memory::size());
        Engine {
            cycle_tracker: CycleTracker::new(config.cycle_tracker_config),
            voices,
            voice_amp: 1.0 / N as f32,
            voice_index: 0,
        }
    }
    
    pub fn process(&mut self, sample: Sample) -> Sample {
        let cycle = self.cycle_tracker.process(sample);

        if cycle.fresh && cycle.length > 0 {
            self.voices[self.voice_index].set_rate(cycle.length);
            self.voice_index = (self.voice_index + 1) % N;
        }

        let mut sample = 0.0;
        for voice in self.voices.iter_mut() {
            sample += self.voice_amp * voice.next_sample();
        }
        sample
    }
}