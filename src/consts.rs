use daisy::audio::FS;
pub use daisy::audio::BLOCK_LENGTH;

pub const SAMPLE_RATE: u32 = FS.to_Hz();
pub const INPUT_BUFFER_SIZE: usize = 2 * 1024;
pub const WINDOW_BUFFER_SIZE: usize = 2 * 1024;
pub const SAMPLING_BUFFER_SIZE: usize = 8 * 1024;
pub const HOP_INTERVAL: usize = 4;
pub const HOP_LIM: usize = INPUT_BUFFER_SIZE / BLOCK_LENGTH;