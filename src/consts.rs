use daisy::audio::FS;
pub use daisy::audio::BLOCK_LENGTH;

pub const SAMPLE_RATE: u32 = FS.to_Hz();
pub const BUFFER_SIZE: usize = 2048;
pub const HOP_INTERVAL: usize = 4;
pub const HOP_LIM: usize = BUFFER_SIZE / BLOCK_LENGTH;