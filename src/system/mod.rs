pub mod memory;

use stm32h7xx_hal::pac::CorePeripherals;
use stm32h7xx_hal::pac::Peripherals as DevicePeripherals;
use stm32h7xx_hal::gpio::{Input, Output, Pin};

pub use rtic_monotonics::systick::prelude::*;
pub use daisy::audio::Interface as AudioInterface;


systick_monotonic!(Mono, 1_000_000);

pub type Led = Pin<'C', 7, Output>;
pub type Gate = Pin<'C', 1, Input>;

pub struct System {
    pub audio_interface: AudioInterface,
    pub gate: Gate,
    pub led: Led
}

impl System {
    pub fn init(mut cp: CorePeripherals, dp: DevicePeripherals) -> Self {
        cp.SCB.enable_icache();
        cp.SCB.enable_dcache(&mut cp.CPUID);

        let board = daisy::Board::take().unwrap();
        let ccdr = daisy::board_freeze_clocks!(board, dp);
        let pins = daisy::board_split_gpios!(board, ccdr, dp);

        let audio_interface = daisy::board_split_audio!(ccdr, pins)
            .spawn()
            .unwrap();
        
        let gate = pins.GPIO.PIN_20.into_floating_input();
        let led = daisy::board_split_leds!(pins).USER;

        Mono::start(cp.SYST, 480_000_000);

        System {
            audio_interface,
            gate,
            led
        }
    }
}