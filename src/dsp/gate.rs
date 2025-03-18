use super::Sample;


#[derive(Default)]
pub enum Gate {
    On,
    #[default]
    Off
}

impl Gate {
    pub fn on(&self) -> bool {
        match self {
            Gate::On  => true,
            Gate::Off => false,
        }
    }

    pub fn off(&self) -> bool {
        match self {
            Gate::On  => false,
            Gate::Off => true,
        }
    }

    pub fn toggle(&mut self) {
        *self = match self {
            Gate::On  => Gate::Off,
            Gate::Off => Gate::On,
        }
    }
}

impl From<Gate> for Sample {
    fn from(value: Gate) -> Self {
        match value {
            Gate::On  => 1.0,
            Gate::Off => 0.0,
        }
    }
}