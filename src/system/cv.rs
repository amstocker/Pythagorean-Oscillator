use daisy::pins::Gpio;
use stm32h7xx_hal::gpio::{
    Analog,
    gpioa::{PA3, PA7},
    gpioc::{PC0, PC4}
};
use stm32h7xx_hal::adc::{Adc, Enabled};
use stm32h7xx_hal::pac::{ADC1, ADC2};
use nb::block;


const CV_LOW: f32 = 0.003;
const CV_HIGH: f32 = 0.97;

pub type Adc1 = Adc<ADC1, Enabled>;
pub type Adc2 = Adc<ADC2, Enabled>;

pub struct CVPins {
    pub cv1: PC0<Analog>,
    pub cv2: PA3<Analog>,
    pub cv3: PC4<Analog>,
    pub cv4: PA7<Analog>
}

#[derive(Default, defmt::Format)]
pub struct CVSample {
    pub cv1: f32,
    pub cv2: f32,
    pub cv3: f32,
    pub cv4: f32
}

impl CVPins {
    pub fn init(gpio: Gpio) -> Self {
        CVPins {
            cv1: gpio.PIN_15.into_analog(),
            cv2: gpio.PIN_16.into_analog(),
            cv3: gpio.PIN_21.into_analog(),
            cv4: gpio.PIN_18.into_analog()
        }
    }

    pub fn sample(&mut self, adc1: &mut Adc1, adc2: &mut Adc2) -> CVSample {
        let mut samples = CVSample::default();
        
        adc1.start_conversion(&mut self.cv1);
        adc2.start_conversion(&mut self.cv2);
        samples.cv1 = scale(block!(adc1.read_sample()).unwrap_or_default(), adc1.slope());
        samples.cv2 = scale(block!(adc2.read_sample()).unwrap_or_default(), adc2.slope());

        adc1.start_conversion(&mut self.cv3);
        adc2.start_conversion(&mut self.cv4);
        samples.cv3 = scale(block!(adc1.read_sample()).unwrap_or_default(), adc1.slope());
        samples.cv4 = scale(block!(adc2.read_sample()).unwrap_or_default(), adc2.slope());

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