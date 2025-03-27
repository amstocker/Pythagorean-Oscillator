use core::f32::consts::PI;
use micromath::F32Ext;


/*
 *  Simple windowing function.
 *  
 *  (See https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows)
 */
const A: f32 = 0.5;

pub fn build_window(window: &mut [f32]) {
    let n = window.len();
    for i in 0..n {
        window[i] = A - (1.0 - A) * ( (2.0 * PI * i as f32) / n as f32 ).cos();
    }
}