#![no_main]
#![no_std]

use prism_firmware as _; // global logger + panicking-behavior + memory layout


#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true)]
mod app {
    use prism_firmware::system::*;
    use prism_firmware::engine::Engine;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        audio_interface: AudioInterface,
        gate: Gate,
        led: Led,
        engine: Engine,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let System {
            audio_interface,
            gate,
            led
        } = System::init(cx.core, cx.device);
        
        input::spawn().ok();
        (
            Shared {},
            Local {
                audio_interface,
                gate,
                led,
                engine: Engine::new()
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

    #[task(local = [gate, led])]
    async fn input(cx: input::Context) {
        loop {
            if !cx.local.gate.is_high() {
                cx.local.led.set_high();
                defmt::debug!("GATE IS HIGH: {}", Mono::now().ticks());
            } else {
                cx.local.led.set_low();
                defmt::debug!("GATE IS LOW: {}", Mono::now().ticks());
            }
            Mono::delay(1.millis()).await;
        }
    }
}