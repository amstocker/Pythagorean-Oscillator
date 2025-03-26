#![no_main]
#![no_std]

use prism_firmware as _; // global logger + panicking-behavior + memory layout


#[rtic::app(
    device = stm32h7xx_hal::pac,
    peripherals = true,
    dispatchers = [EXTI0, EXTI1]
)]
mod app {
    use fugit::{MicrosDuration, Hertz, RateExtU32};
    use rtic_sync::{make_channel, make_signal};
    use rtic_sync::signal::{Signal, SignalReader, SignalWriter};
    use rtic_sync::channel::{Sender, Receiver};
    
    use prism_firmware::consts::{INPUT_BUFFER_SIZE, *};
    use prism_firmware::system::*;
    use prism_firmware::engine::{Analyzer, Prism};


    type WindowBuffer = &'static mut [f32];

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        audio_interface: AudioInterface,
        audio_input_buffer: WindowBuffer,
        audio_input_index: usize,
        analyzer: Analyzer,
        prism: Prism,
        input: Input,
        input_tx: SignalWriter<'static, InputSample>,
        input_rx: SignalReader<'static, InputSample>,
        recent_input_sample: InputSample,
        window_tx: Sender<'static, WindowBuffer, 4>,
        window_rx: Receiver<'static, WindowBuffer, 4>
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let System {
            audio_interface,
            input
        } = System::init(cx.core, cx.device);

        let (input_tx, input_rx) = make_signal!(InputSample);
        let (window_tx, window_rx) = make_channel!(WindowBuffer, 4);
        
        let local = Local {
            audio_interface,
            audio_input_buffer: memory::allocate_f32_buffer(
                INPUT_BUFFER_SIZE + WINDOW_BUFFER_SIZE
            ).unwrap(),
            audio_input_index: WINDOW_BUFFER_SIZE,
            analyzer: Analyzer::init(),
            prism: Prism::init(),
            input,
            input_tx,
            input_rx,
            recent_input_sample: InputSample::default(),
            window_tx,
            window_rx
        };

        input::spawn(100.Hz()).unwrap();
        analyze::spawn().unwrap();

        defmt::trace!("Finished init (free memory: {} / {})", memory::capacity(), memory::size());
        (Shared {}, local)
    }


    #[task(
        binds = DMA1_STR1,
        priority = 5,
        local = [
            audio_interface,
            audio_input_buffer,
            audio_input_index,
            prism,
            input_rx,
            recent_input_sample,
            window_tx
        ]
    )]
    fn dsp(cx: dsp::Context) {
        let dsp::LocalResources {
            audio_interface,
            audio_input_buffer,
            audio_input_index,
            prism,
            input_rx,
            recent_input_sample,
            window_tx, ..
        } = cx.local;

        if let Some(input_sample) = input_rx.try_read() {
            *recent_input_sample = input_sample;
        }

        audio_interface.handle_interrupt_dma1_str1(|audio_buffer| {
            for i in 0..BLOCK_LENGTH {
                let sample = audio_buffer[i].0;
                audio_input_buffer[*audio_input_index + i] = sample;
                audio_input_buffer[(*audio_input_index + i) % INPUT_BUFFER_SIZE] = sample;
            }
            *audio_input_index += BLOCK_LENGTH;

            if *audio_input_index % WINDOW_HOP == 0 {
                let window_start = (*audio_input_index)
                    - WINDOW_BUFFER_SIZE
                    % INPUT_BUFFER_SIZE;

                let window = unsafe {
                    core::slice::from_raw_parts_mut(
                        &mut audio_input_buffer[window_start],
                        WINDOW_BUFFER_SIZE
                    )
                };

                window_tx.try_send(window).unwrap();
            }

            if *audio_input_index == INPUT_BUFFER_SIZE + WINDOW_BUFFER_SIZE {
                *audio_input_index = WINDOW_BUFFER_SIZE;
            }
        })
        .unwrap();
    }


    #[task(
        priority = 3,
        local = [
            analyzer,
            window_rx
        ]
    )]
    async fn analyze(cx: analyze::Context) {
        let analyze::LocalResources {
            analyzer,
            window_rx, ..
        } = cx.local;

        while let Ok(window) = window_rx.recv().await {
            let frequency = analyzer.process(window);
            defmt::debug!("\tfreq={}", frequency);
        }
    }


    #[task(
        priority = 1,
        local = [
            input,
            input_tx
        ]
    )]
    async fn input(cx: input::Context, sample_rate: Hertz<u32>) {
        let input::LocalResources {
            input,
            input_tx, ..
        } = cx.local;

        let delay: MicrosDuration<u32> = sample_rate.into_duration();
        defmt::debug!("delay={}", delay);
        loop {
            let now = Mono::now();

            input_tx.write(input.sample());

            Mono::delay_until(
                now.checked_add_duration(delay).unwrap()
            ).await;
        }
    }
}