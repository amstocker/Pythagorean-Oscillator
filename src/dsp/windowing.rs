use core::f32::consts::PI;
use micromath::F32Ext;

use crate::system::memory;


pub fn build_window(len: usize) -> &'static [f32] {
    let window = memory::allocate_buffer(len).unwrap();
    for i in 0..len {
        window[i] = 0.5 - 0.5 * ( (2.0 * PI * i as f32) / len as f32 ).cos();
    }
    window
}