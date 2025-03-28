use daisy::pins::Gpio;
use stm32h7xx_hal::{gpio::{
    gpioa::{PA3, PA7},
    gpioc::{PC0, PC4},
    Analog,
    Pin
}, hal::adc::Channel};
use stm32h7xx_hal::adc::{Adc, Enabled};
use stm32h7xx_hal::pac::{ADC1, ADC2};
use nb::block;

use crate::dsp::lpf::LowPassFilter;


const CV_LOW: f32 = 0.002;
const CV_HIGH: f32 = 0.970;
const CV_LPF_FREQ: f32 = 10_000.0;

pub type Adc1 = Adc<ADC1, Enabled>;
pub type Adc2 = Adc<ADC2, Enabled>;

pub struct Input {
    adc1: Adc1,
    adc2: Adc2,
    pub cv1: PC0<Analog>,
    pub cv2: PA3<Analog>,
    pub cv3: PC4<Analog>,
    pub cv4: PA7<Analog>
}

#[derive(Default, Clone, Copy, defmt::Format)]
pub struct InputSample {
    pub cv1: f32,
    pub cv2: f32,
    pub cv3: f32,
    pub cv4: f32
}

impl Input {
    pub fn init(gpio: Gpio, adc1: Adc1, adc2: Adc2) -> Self {
        Input {
            adc1,
            adc2,
            cv1: gpio.PIN_15.into_analog(),
            cv2: gpio.PIN_16.into_analog(),
            cv3: gpio.PIN_21.into_analog(),
            cv4: gpio.PIN_18.into_analog()
        }
    }

    pub fn sample(&mut self) -> InputSample {
        let mut samples = InputSample::default();
        
        self.adc1.start_conversion(&mut self.cv1);
        self.adc2.start_conversion(&mut self.cv2);
        samples.cv1 = 
            scale(block!(self.adc1.read_sample()).unwrap_or_default(), self.adc1.slope());
        samples.cv2 =
            scale(block!(self.adc2.read_sample()).unwrap_or_default(), self.adc2.slope());

        self.adc1.start_conversion(&mut self.cv3);
        self.adc2.start_conversion(&mut self.cv4);
        samples.cv3 =
            scale(block!(self.adc1.read_sample()).unwrap_or_default(), self.adc1.slope());
        samples.cv4 =
            scale(block!(self.adc2.read_sample()).unwrap_or_default(), self.adc2.slope());

        samples        
    }
}

fn scale(sample: u32, slope: u32) -> f32 {
    let sample = sample as f32;
    let slope = slope as f32;
    let actual = (slope - sample) / slope;
    let scaled = (actual - CV_LOW) / (CV_HIGH - CV_LOW);

    scaled.clamp(0.0, 1.0)
}