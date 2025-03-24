#![no_main]
#![no_std]

use prism_firmware as _; // global logger + panicking-behavior + memory layout


#[rtic::app(
    device = stm32h7xx_hal::pac,
    peripherals = true,
    dispatchers = [EXTI0]
)]
mod app {
    use prism_firmware::system::{memory, *};


    use daisy::audio::BLOCK_LENGTH;
    use prism_firmware::dsp::windowing;

    const BUFFER_SIZE: usize = 2048;
    const HOP_INTERVAL: usize = 4;  // In number of blocks
    const HOP_LIM: usize = BUFFER_SIZE / BLOCK_LENGTH;


    #[shared]
    struct Shared {
        window_buffer: &'static mut [f32]
    }

    #[local]
    struct Local {
        audio_interface: AudioInterface,
        main_buffer: &'static mut [f32],
        hop_counter: usize,
        windowing_func: &'static [f32],
        phase_data: &'static mut [f32],
        frequency: f32
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let System {
            audio_interface,
            ..
        } = System::init(cx.core, cx.device);
        
        let shared = Shared {
            window_buffer: memory::allocate_buffer(2 * BUFFER_SIZE).unwrap()
        };

        let local = Local {
            audio_interface,
            main_buffer: memory::allocate_buffer(BUFFER_SIZE).unwrap(),
            hop_counter: 0,
            windowing_func: windowing::build_window(BUFFER_SIZE),
            phase_data: memory::allocate_buffer(BUFFER_SIZE).unwrap(),
            frequency: 0.0
        };

        defmt::debug!("Finished init (free memory: {} / {} kB)", memory::capacity() / 1024, memory::size() / 1024);

        (shared, local)
    }

    #[task(
        binds = DMA1_STR1,
        priority = 5,
        local = [
            audio_interface,
            main_buffer,
            hop_counter
        ],
        shared = [
            window_buffer
        ]
    )]
    fn dsp(cx: dsp::Context) {
        let dsp::LocalResources {
            audio_interface,
            main_buffer,
            hop_counter, ..
        } = cx.local;

        let dsp::SharedResources {
            mut window_buffer, ..
        } = cx.shared;

        audio_interface.handle_interrupt_dma1_str1(|audio_buffer| {
            let start = *hop_counter * BLOCK_LENGTH;
            for i in 0..BLOCK_LENGTH {
                main_buffer[start + i] = audio_buffer[i].0;
            }

            *hop_counter = (*hop_counter + 1) % HOP_LIM;
            
            if *hop_counter % HOP_INTERVAL == 0 {
                window_buffer.lock(|window_buffer| {
                    let end = BUFFER_SIZE - start;
                    for i in 0..end {
                        window_buffer[2 * i] = main_buffer[start + i];
                        window_buffer[2 * i + 1] = 0.0;
                    }
                    for i in 0..start {
                        window_buffer[2 * (end + i)] = main_buffer[i];
                        window_buffer[2 * (end + i) + 1] = 0.0;
                    }
                });

                analyze::spawn().ok();
            }
        })
        .unwrap();
    }

    #[task(
        priority = 1,
        local = [
            windowing_func,
            phase_data,
            frequency
        ],
        shared = [
            window_buffer
        ]
    )]
    async fn analyze(cx: analyze::Context) {
        let analyze::LocalResources {
            windowing_func,
            phase_data,
            frequency, ..
        } = cx.local;

        let analyze::SharedResources {
            mut window_buffer, ..
        } = cx.shared;


        use microfft::{Complex32, complex::cfft_2048};
        use micromath::F32Ext;
        use core::f32::consts::PI;
        use daisy::audio::FS;

        window_buffer.lock(|window_buffer| {
            for i in 0..BUFFER_SIZE {
                window_buffer[2 * i] *= windowing_func[i];
            }

            let samples = {
                let slice = unsafe {
                    let ptr = window_buffer.as_mut_ptr().cast::<Complex32>();
                    core::slice::from_raw_parts_mut(ptr, BUFFER_SIZE)
                };
                slice.try_into().unwrap()
            };

            let spectrum = cfft_2048(samples);

            let mut max_index = 0;
            let mut max_norm_sq = 0.0;
            let mut max_prev_phase = 0.0;
            for i in 0..(BUFFER_SIZE / 2) {
                let Complex32 { re, im} = spectrum[i];
                let norm_sq = re * re + im * im;
                if norm_sq > max_norm_sq {
                    max_norm_sq = norm_sq;
                    max_index = i;
                    max_prev_phase = phase_data[i];
                }
                phase_data[i] = im.atan2(re);
            }

            let max_freq = (max_index as f32 / BUFFER_SIZE as f32) * FS.to_Hz() as f32; 

            let dt = 2.0 * PI * ((HOP_INTERVAL * BLOCK_LENGTH) as f32 / FS.to_Hz() as f32);
            let dp = phase_data[max_index] - max_prev_phase;
            let mut p = 0.0;
            let mut f_prev = 0.0;
            loop {
                let f = (dp + p) / dt;
                if f > max_freq {
                    *frequency = if f - max_freq < max_freq - f_prev {
                        f
                    } else {
                        f_prev
                    };
                    break;
                }
                f_prev = f;
                p += 2.0 * PI;
            }

            defmt::debug!("freq = {}", *frequency);
        });
    }
}