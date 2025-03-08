#![no_main]
#![no_std]

use prism_firmware as _; // global logger + panicking-behavior + memory layout



// This is just a placeholder TimeSource. In a real world application
// one would probably use the RTC to provide time.
pub struct TimeSource;

impl embedded_sdmmc::TimeSource for TimeSource {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        embedded_sdmmc::Timestamp {
            year_since_1970: 0,
            zero_indexed_month: 0,
            zero_indexed_day: 0,
            hours: 0,
            minutes: 0,
            seconds: 0,
        }
    }
}



#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true)]
mod app {
    use stm32h7xx_hal::{
        prelude::*,
        gpio::Speed,
        sdmmc::{SdCard, Sdmmc},
    };
    use embedded_sdmmc::{sdcard, Mode, SdCard as SD, VolumeIdx, VolumeManager};
    use daisy::audio::Interface;
    use defmt::debug;

    use crate::TimeSource;


    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        audio_interface: Interface,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let mut cp = cx.core;
        let dp = cx.device;

        // Using caches should provide a major performance boost.
        cp.SCB.enable_icache();
        // NOTE: Data caching requires cache management around all use of DMA.
        // This crate already handles that for audio processing.
        cp.SCB.enable_dcache(&mut cp.CPUID);

        let board = daisy::Board::take().unwrap();

        let ccdr = daisy::board_freeze_clocks!(board, dp);
        let pins = daisy::board_split_gpios!(board, ccdr, dp);
        let mut led_user = daisy::board_split_leds!(pins).USER;
        let one_second = ccdr.clocks.sys_ck().to_Hz();
        
        let audio_interface = daisy::board_split_audio!(ccdr, pins);
        let audio_interface = audio_interface.spawn().unwrap();


        // SD Card
        let (clk, cmd, d0, d1, d2, d3) = (
            pins.GPIO.PIN_6,
            pins.GPIO.PIN_5,
            pins.GPIO.PIN_4,
            pins.GPIO.PIN_3,
            pins.GPIO.PIN_2,
            pins.GPIO.PIN_1,
        );

        let clk = clk
            .into_alternate::<12>()
            .internal_pull_up(false)
            .speed(Speed::VeryHigh);
        let clk = clk
            .into_alternate()
            .internal_pull_up(false)
            .speed(Speed::VeryHigh);
        let cmd = cmd
            .into_alternate()
            .internal_pull_up(true)
            .speed(Speed::VeryHigh);
        let d0 = d0
            .into_alternate()
            .internal_pull_up(true)
            .speed(Speed::VeryHigh);
        let d1 = d1
            .into_alternate()
            .internal_pull_up(true)
            .speed(Speed::VeryHigh);
        let d2 = d2
            .into_alternate()
            .internal_pull_up(true)
            .speed(Speed::VeryHigh);
        let d3 = d3
            .into_alternate()
            .internal_pull_up(true)
            .speed(Speed::VeryHigh);

        let mut sdmmc: Sdmmc<_, SdCard> = dp.SDMMC1.sdmmc(
            (clk, cmd, d0, d1, d2, d3),
            ccdr.peripheral.SDMMC1,
            &ccdr.clocks,
        );

        let bus_frequency = 2.MHz();

        while sdmmc.init(bus_frequency).is_err() {
            led_user.toggle();
            cortex_m::asm::delay(one_second / 8);
        }

        let block_device = sdmmc.sdmmc_block_device();
        let mut volume_mgr = VolumeManager::new(block_device, TimeSource);
        let volume0 = volume_mgr.get_volume(VolumeIdx(0)).unwrap();
        let root_dir = volume_mgr.open_root_dir(&volume0).unwrap();
        volume_mgr.iterate_dir(&volume0, &root_dir, |entry| {
            debug!("files: {}", entry.size);
        }).unwrap();

        debug!("Finished init.");

        (
            Shared {},
            Local {
                audio_interface
            }
        )
    }

    #[task(binds = DMA1_STR1, local = [audio_interface])]
    fn dsp(cx: dsp::Context) {
        let audio_interface = cx.local.audio_interface;

        audio_interface
            .handle_interrupt_dma1_str1(|audio_buffer| {
                for frame in audio_buffer {
                    let (left, right) = *frame;
                    *frame = (right * 0.8, left * 0.8);
                }
            })
            .unwrap();
    }
}