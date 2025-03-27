#![no_main]
#![no_std]

use prism_firmware as _; // global logger + panicking-behavior + memory layout


#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true)]
mod app {
    use fugit::{MicrosDuration, Hertz, RateExtU32};
    use rtic_sync::{make_channel, make_signal};
    use rtic_sync::signal::{Signal, SignalReader, SignalWriter};
    use rtic_sync::channel::{Sender, Receiver};
    
    use prism_firmware::consts::{INPUT_BUFFER_SIZE, *};
    use prism_firmware::system::*;
    use prism_firmware::engine::Analyzer;


    type Buffer = &'static mut [f32];

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        audio_interface: AudioInterface,
        audio_input_buffer: [f32; INPUT_BUFFER_SIZE + WINDOW_BUFFER_SIZE],
        audio_input_index: usize,
        analyzer: Analyzer,
        input: Input,
        input_tx: SignalWriter<'static, InputSample>,
        input_rx: SignalReader<'static, InputSample>,
        recent_input_sample: InputSample,
        window_tx: Sender<'static, Buffer, 1>,
        window_rx: Receiver<'static, Buffer, 1>,
        warmup_counter: usize
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let System {
            audio_interface,
            input
        } = System::init(cx.core, cx.device);

        let (input_tx, input_rx) = make_signal!(InputSample);
        let (window_tx, window_rx) = make_channel!(Buffer, 1);
        
        let local = Local {
            audio_interface,
            audio_input_buffer: [0.0; INPUT_BUFFER_SIZE + WINDOW_BUFFER_SIZE],
            audio_input_index: WINDOW_BUFFER_SIZE,
            analyzer: Analyzer::init(),
            input,
            input_tx,
            input_rx,
            recent_input_sample: InputSample::default(),
            window_tx,
            window_rx,
            warmup_counter: INPUT_BUFFER_SIZE / BLOCK_LENGTH
        };

        input::spawn(100.Hz()).unwrap();
        analyze::spawn().unwrap();

        defmt::trace!("Finished init");
        (Shared {}, local)
    }


    #[task(
        binds = DMA1_STR1,
        priority = 3,
        local = [
            audio_interface,
            audio_input_buffer,
            audio_input_index,
            input_rx,
            recent_input_sample,
            window_tx,
            warmup_counter
        ]
    )]
    fn dsp(cx: dsp::Context) {
        let dsp::LocalResources {
            audio_interface,
            audio_input_buffer,
            audio_input_index,
            input_rx,
            recent_input_sample,
            window_tx,
            warmup_counter, ..
        } = cx.local;

        audio_interface.handle_interrupt_dma1_str1(|audio_buffer| {
            if *audio_input_index < INPUT_BUFFER_SIZE {
                for i in 0..BLOCK_LENGTH {
                    audio_input_buffer[*audio_input_index + i] = audio_buffer[i].0;
                    audio_buffer[i] = (0.0, 0.0);
                }
            } else {
                for i in 0..BLOCK_LENGTH {
                    let x = audio_buffer[i].0;
                    audio_input_buffer[*audio_input_index + i] = x;
                    audio_input_buffer[*audio_input_index + i - INPUT_BUFFER_SIZE] = x;
                    audio_buffer[i] = (0.0, 0.0);
                }
            }
        }).unwrap();

        *audio_input_index += BLOCK_LENGTH;
        if *audio_input_index == INPUT_BUFFER_SIZE + WINDOW_BUFFER_SIZE {
            *audio_input_index = WINDOW_BUFFER_SIZE;
        }

        if *audio_input_index % WINDOW_HOP == 0 && *warmup_counter == 0 {
            let window_start = *audio_input_index - WINDOW_BUFFER_SIZE;

            // Safety: Input buffer should be large enough that window slice
            //         is never written to while analyzer is copying the window
            //         to its internal buffer.
            let window = unsafe {
                core::slice::from_raw_parts_mut(
                    &mut audio_input_buffer[window_start],
                    WINDOW_BUFFER_SIZE
                )
            };

            window_tx.try_send(window).unwrap();
        }

        if let Some(input_sample) = input_rx.try_read() {
            *recent_input_sample = input_sample;
        }

        if *warmup_counter > 0 {
            *warmup_counter -= 1;
        };        
    }


    #[task(local = [analyzer, window_rx])]
    async fn analyze(cx: analyze::Context) {
        let analyze::LocalResources {
            analyzer,
            window_rx, ..
        } = cx.local;

        while let Ok(window) = window_rx.recv().await {
            let frequency = analyzer.process(window);
            defmt::debug!("freq = {}", frequency);
        }
    }

    #[task(local = [input, input_tx])]
    async fn input(cx: input::Context, sample_rate: Hertz<u32>) {
        let input::LocalResources {
            input,
            input_tx, ..
        } = cx.local;

        let delay: MicrosDuration<u32> = sample_rate.into_duration();
        loop {
            input_tx.write(input.sample());
            
            Mono::delay(delay).await;
        }
    }
}