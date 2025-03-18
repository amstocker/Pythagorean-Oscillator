#![no_main]
#![no_std]

use prism_firmware as _; // global logger + panicking-behavior + memory layout


#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true)]
mod app {
    use daisy::audio::Interface;

    use prism_firmware::engine::Engine;
    use prism_firmware::config::{NUM_VOICES, ENGINE_CONFIG};


    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        audio_interface: Interface,
        engine: Engine<NUM_VOICES>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut cp = cx.core;
        let dp = cx.device;

        cp.SCB.enable_icache();
        cp.SCB.enable_dcache(&mut cp.CPUID);

        let board = daisy::Board::take().unwrap();

        let ccdr = daisy::board_freeze_clocks!(board, dp);
        let pins = daisy::board_split_gpios!(board, ccdr, dp);
        //let mut led_user = daisy::board_split_leds!(pins).USER;
        //let one_second = ccdr.clocks.sys_ck().to_Hz();
        
        let audio_interface = daisy::board_split_audio!(ccdr, pins);
        let audio_interface = audio_interface.spawn().unwrap();

        (
            Shared {},
            Local {
                audio_interface,
                engine: Engine::new(ENGINE_CONFIG)
            }
        )
    }

    #[task(binds = DMA1_STR1, local = [audio_interface, engine])]
    fn audio(cx: audio::Context) {
        let audio_interface = cx.local.audio_interface;
        let engine = cx.local.engine;

        audio_interface
            .handle_interrupt_dma1_str1(|audio_buffer| {
                for frame in audio_buffer {
                    *frame = (frame.0, engine.process(frame.0));
                }
            })
            .unwrap();
    }
}