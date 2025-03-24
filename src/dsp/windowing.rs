use core::f32::consts::PI;
use micromath::F32Ext;

use crate::system::memory;


/*
 *  Simple windowing function.
 *  
 *  (See https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows)
 */
const A: f32 = 0.5;

pub fn build_windowing_func(len: usize) -> &'static [f32] {
    let window = memory::allocate_f32_buffer(len).unwrap();
    for i in 0..len {
        window[i] = A - (1.0 - A) * ( (2.0 * PI * i as f32) / len as f32 ).cos();
    }
    window
}