use crate::system::AudioInterface;

pub struct Reader {
    audio_interface: AudioInterface,
    hop_counter: usize,
}

impl Reader {
    pub fn init(audio_interface: AudioInterface) -> Self {
        Reader {
            audio_interface,
            hop_counter: 0
        }
    }
}