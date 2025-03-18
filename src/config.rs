use crate::dsp::cycle_detector::Config as CycleDetectorConfig;
use crate::engine::Config as EngineConfig;
use crate::engine::CycleTrackerConfig;
use crate::engine::VoiceConfig;

pub const NUM_VOICES: usize = 8;

pub const ENGINE_CONFIG: EngineConfig = EngineConfig {
    voice_config: VoiceConfig {
        buffer_size: 2048,
    },
    cycle_tracker_config: CycleTrackerConfig {
        buffer_size: 8192,
        cycle_detector_config: CycleDetectorConfig {
            lpf_decay: 0.005,
            env_rise: 0.1,
            env_fall: 0.0001,
        },
    },
};