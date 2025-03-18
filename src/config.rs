use crate::dsp;
use crate::engine;


pub const ENGINE_CONFIG: engine::Config = engine::Config {
    num_voices: 8,
    voice_config: engine::voice::Config {
        buffer_size: 2048,
    },
    tracker_config: engine::tracker::Config {
        buffer_size: 8192,
        cycle_detector_config: dsp::cycle_detector::Config {
            lpf_decay: 0.005,
            env_rise: 0.1,
            env_fall: 0.0001,
        },
    },
};