use core::mem::MaybeUninit;


const MEMORY_BYTES: usize = 512 * 1024;
const MEMORY_SIZE: usize = MEMORY_BYTES / core::mem::size_of::<f32>();

#[unsafe(link_section = ".sram")]
static mut MEMORY: [MaybeUninit<f32>; MEMORY_SIZE] = [MaybeUninit::uninit(); MEMORY_SIZE];


/*
 *  Simple allocator for f32 buffers.  
 */
pub fn allocate_buffer(len: usize) -> Option<&'static mut [f32]> {
    static mut INDEX: usize = 0;
    
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