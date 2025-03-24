use core::mem::MaybeUninit;

use microfft::Complex32;


const MEMORY_BYTES: usize = 512 * 1024;
const MEMORY_SIZE: usize = MEMORY_BYTES / core::mem::size_of::<f32>();

#[unsafe(link_section = ".sram")]
static mut MEMORY: [MaybeUninit<f32>; MEMORY_SIZE] = [MaybeUninit::uninit(); MEMORY_SIZE];


/*
 *  Simple allocator for f32 buffers.  
 */
static mut INDEX: usize = 0;

pub fn allocate_f32_buffer(len: usize) -> Option<&'static mut [f32]> {
    let buffer = unsafe {
        if INDEX + len > MEMORY_SIZE {
            return None;
        } else {
            let buffer = core::slice::from_raw_parts_mut(&mut MEMORY[INDEX], len);
            INDEX += len;
            buffer
        }
    };

    for x in buffer.iter_mut() {
        x.write(0.0);
    }

    unsafe { Some(core::mem::transmute(buffer)) }
}

pub fn allocate_complex32_buffer(len: usize) -> Option<&'static mut [Complex32]> {
    allocate_f32_buffer(2 * len).map(|buffer|
        unsafe {
            let data = buffer.as_mut_ptr().cast::<Complex32>();
            core::slice::from_raw_parts_mut(data, len)
        }
    )
}

pub fn size() -> usize {
    MEMORY_SIZE
}

pub fn capacity() -> usize {
    unsafe { MEMORY_SIZE - INDEX }
}