#![no_main]
#![no_std]

use prism_firmware as _; // global logger + panicking-behavior + memory layout


#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true)]
mod app {
    use daisy::audio::Interface;
    use defmt::debug;

    use prism_firmware::{dsp::Processor, engine::CycleTracker};
    


    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        audio_interface: Interface,

        // TODO: put const somewhere else?
        cycle_tracker: CycleTracker<4096>,

        phase: f32
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
        let mut led_user = daisy::board_split_leds!(pins).USER;
        let one_second = ccdr.clocks.sys_ck().to_Hz();
        
        let audio_interface = daisy::board_split_audio!(ccdr, pins);
        let audio_interface = audio_interface.spawn().unwrap();

        debug!("Finished init.");
        (
            Shared {},
            Local {
                audio_interface,
                cycle_tracker: CycleTracker::new(),
                phase: 0.0
            }
        )
    }

    #[task(binds = DMA1_STR1, local = [audio_interface, cycle_tracker, phase])]
    fn audio(cx: audio::Context) {
        use micromath::F32Ext;
        use core::f32::consts::PI;

        let audio_interface = cx.local.audio_interface;
        let cycle_tracker = cx.local.cycle_tracker;
        let phase = cx.local.phase;

        audio_interface
            .handle_interrupt_dma1_str1(|audio_buffer| {
                for frame in audio_buffer {
                    let cycle = cycle_tracker.process(frame.0);
                    if cycle.length() > 0 {
                        *phase += 1.0 / cycle.length() as f32;
                        if *phase > 1.0 {
                            *phase -= 1.0;
                        }
                    }
                    *frame = (frame.0, (*phase * 2.0 * PI).sin());
                }
            })
            .unwrap();
    }
}