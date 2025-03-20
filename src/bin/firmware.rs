#![no_main]
#![no_std]

use prism_firmware as _; // global logger + panicking-behavior + memory layout


#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true)]
mod app {
    use stm32h7xx_hal::gpio::{Input, Output, Pin};
    use rtic_monotonics::systick::prelude::*;
    use daisy::audio::Interface;

    use prism_firmware::engine::Engine;
    
    
    systick_monotonic!(Mono, 1000);


    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        audio_interface: Interface,
        engine: Engine,
        gate1: Pin<'C', 1, Input>,
        led: Pin<'C', 7, Output>
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
        let led_user = daisy::board_split_leds!(pins).USER;

        // Gate1 input on 
        let gate1 = pins.GPIO.PIN_20.into_floating_input();
        
        let audio_interface = daisy::board_split_audio!(ccdr, pins);
        let audio_interface = audio_interface.spawn().unwrap();

        Mono::start(cp.SYST, 480_000_000);

        input::spawn().ok();
        (
            Shared {},
            Local {
                audio_interface,
                engine: Engine::new(),
                gate1,
                led: led_user
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

    #[task(local = [gate1, led])]
    async fn input(cx: input::Context) {
        loop {
            if !cx.local.gate1.is_high() {
                cx.local.led.set_high();
                defmt::debug!("GATE IS HIGH");
            } else {
                cx.local.led.set_low();
                defmt::debug!("GATE IS LOW");
            }
            Mono::delay(1.millis()).await;
        }
    }
}