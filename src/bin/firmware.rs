#![no_main]
#![no_std]

use prism_firmware as _; // global logger + panicking-behavior + memory layout


#[rtic::app(
    device = stm32h7xx_hal::pac,
    peripherals = true,
    dispatchers = [EXTI0]
)]
mod app {
    use prism_firmware::consts::*;
    use prism_firmware::system::*;
    use prism_firmware::engine::Analyzer;


    #[shared]
    struct Shared {
        window_buffer: &'static mut [f32]
    }

    #[local]
    struct Local {
        audio_interface: AudioInterface,
        main_buffer: &'static mut [f32],
        hop_counter: usize,
        analyzer: Analyzer
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
            analyzer: Analyzer::init()
        };

        defmt::trace!("Finished init (free memory: {} / {} kB)", memory::capacity() / 1024, memory::size() / 1024);

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
        local = [analyzer],
        shared = [window_buffer]
    )]
    async fn analyze(cx: analyze::Context) {
        let analyzer = cx.local.analyzer;
        let mut window_buffer = cx.shared.window_buffer;

        window_buffer.lock(|window_buffer| {
            analyzer.process(window_buffer);     
        });
    }
}