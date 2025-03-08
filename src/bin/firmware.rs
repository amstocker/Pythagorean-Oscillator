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


const WAV_FILENAMES: [&str; 16] = [
    "0-0-0-0.wav",
    "1-0-0-0.wav",
    "0-1-0-0.wav",
    "1-1-0-0.wav",
    "0-0-1-0.wav",
    "1-0-1-0.wav",
    "0-1-1-0.wav",
    "1-1-1-0.wav",
    "0-0-0-1.wav",
    "1-0-0-1.wav",
    "0-1-0-1.wav",
    "1-1-0-1.wav",
    "0-0-1-1.wav",
    "1-0-1-1.wav",
    "0-1-1-1.wav",
    "1-1-1-1.wav"
];



#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true)]
mod app {
    use stm32h7xx_hal::{
        prelude::*,
        gpio::Speed,
        sdmmc::{SdCard, Sdmmc},
    };
    use embedded_sdmmc::{VolumeIdx, VolumeManager, Mode};
    use daisy::audio::{Interface, BLOCK_LENGTH};
    use defmt::debug;

    use crate::{WAV_FILENAMES, TimeSource};


    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        audio_interface: Interface,
        raw_memory: &'static [i16],
        frame_counter: usize,
        frame_max: usize
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

        let sdram = daisy::board_split_sdram!(cp, dp, ccdr, pins);

        let raw_memory = unsafe {
            let ram_items = sdram.size() / core::mem::size_of::<i16>();
            let ram_ptr = sdram.base_address as *mut i16;
            core::slice::from_raw_parts_mut(ram_ptr, ram_items)
        };


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
        let mut volume0 = volume_mgr.get_volume(VolumeIdx(0)).unwrap();
        let root_dir = volume_mgr.open_root_dir(&volume0).unwrap();

        const HEADER_LEN: usize = 44;
        const DATA_LEN: usize = 2 * 256 * 8 * 8;
        let mut shift = 0;
        for filename in WAV_FILENAMES {
            let mut file = volume_mgr.open_file_in_dir(&mut volume0, &root_dir, filename, Mode::ReadOnly).unwrap();
            file.seek_from_start(0).unwrap();
            let mut buffer = [0u8; HEADER_LEN + DATA_LEN];
            volume_mgr
                .read(&volume0, &mut file, &mut buffer)
                .unwrap();

            raw_memory[shift..shift+(DATA_LEN/2)].copy_from_slice(
                bytemuck::cast_slice(&buffer[HEADER_LEN..HEADER_LEN+DATA_LEN])
            );
            shift += DATA_LEN / 2;

            volume_mgr.close_file(&volume0, file).unwrap();
        }

        debug!("Finished init.");
        (
            Shared {},
            Local {
                audio_interface,
                raw_memory,
                frame_counter: 0,
                frame_max: shift
            }
        )
    }

    #[task(binds = DMA1_STR1, local = [audio_interface, raw_memory, frame_counter, frame_max])]
    fn dsp(cx: dsp::Context) {
        let audio_interface = cx.local.audio_interface;
        let raw_memory = cx.local.raw_memory;
        let frame_counter = cx.local.frame_counter;

        let mut buffer = [0.0; BLOCK_LENGTH];
        for i in 0..BLOCK_LENGTH {
            buffer[i] = raw_memory[*frame_counter+i] as f32 / 32767 as f32;
        }
        *frame_counter = (*frame_counter + BLOCK_LENGTH) % *cx.local.frame_max;

        audio_interface
            .handle_interrupt_dma1_str1(|audio_buffer| {
                for (frame, sample) in audio_buffer.iter_mut().zip(buffer.into_iter()) {
                    *frame = (0.5 * sample, 0.5 * sample);
                }
            })
            .unwrap();
    }
}