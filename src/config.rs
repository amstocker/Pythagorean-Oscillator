pub const MAIN_BUFFER_SIZE: usize = 4096;

pub const NUM_VOICES: usize = 4;
pub const VOICE_BUFFER_SIZE: usize = 2048;

pub const CYCLE_DETECT_ENV_RISE: f32 = 0.1;

// TODO: This param should be configurable via CV.
pub const CYCLE_DETECT_ENV_FALL: f32 = 0.001;