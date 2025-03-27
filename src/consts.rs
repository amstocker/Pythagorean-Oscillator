use daisy::audio::FS;
pub use daisy::audio::BLOCK_LENGTH;

pub const SAMPLE_RATE: u32 = FS.to_Hz();
pub const INPUT_BUFFER_SIZE: usize = 4 * 1024;

// Must match FFT size (in Cargo.toml and analyzer.rs)
pub const WINDOW_BUFFER_SIZE: usize = 1024;
pub const WINDOW_HOP: usize = 8 * BLOCK_LENGTH;