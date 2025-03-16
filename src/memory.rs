use core::mem::MaybeUninit;


const MEMORY_BYTES: usize = 512 * 1024;
const MEMORY_SIZE: usize = MEMORY_BYTES / core::mem::size_of::<f32>();

#[unsafe(link_section = ".sram")]
static mut MEMORY: [MaybeUninit<f32>; MEMORY_SIZE] = [MaybeUninit::uninit(); MEMORY_SIZE];


pub fn allocate_buffer(len: usize) -> Option<&'static mut [f32]> {
    static mut INDEX: usize = 0;
    unsafe {
        if INDEX + len > MEMORY_SIZE {
            None
        } else {
            let buffer = core::slice::from_raw_parts_mut(&mut MEMORY[INDEX], len);
            for i in 0..len {
                buffer[i].write(0.0);
            }
            INDEX += len;
            Some(core::mem::transmute(buffer))
        }
    }
}