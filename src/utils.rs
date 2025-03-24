use crate::system::*;


pub struct Timer {
    start: u32
}

impl Timer {
    pub fn start() -> Self {
        Timer {
            start: Mono::now().ticks()
        }
    }

    pub fn end(&self) -> u32 {
        let now = Mono::now().ticks();
        now - self.start
    }
}