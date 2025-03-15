
const MEMORY_BYTES: usize = 512 * 1024;
const MEMORY_SIZE: usize = MEMORY_BYTES / core::mem::size_of::<f32>();

// Warning: This is not actually initialized to zero!
#[unsafe(link_section = ".sram")]
static mut MEMORY: [f32; MEMORY_SIZE] = [0.0; MEMORY_SIZE];


pub fn allocate_buffer(len: usize) -> Option<&'static mut [f32]> {
    static mut INDEX: usize = 0;
    unsafe {
        if INDEX + len > MEMORY_SIZE {
            None
        } else {
            let buffer = core::slice::from_raw_parts_mut(&mut MEMORY[INDEX], len);
            INDEX += len;
            Some(buffer)
        }
    }
}