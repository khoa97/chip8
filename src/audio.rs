use rodio::source::{SineWave, Source};
use rodio::{OutputStream, OutputStreamHandle, Sink};

// use std::time::Duration;
pub struct Audio {
    sink: Sink,
}

impl Audio {
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        return Audio { sink };
    }

    pub fn beep(&self) {
        let source = SineWave::new(440.0).amplify(0.20);

        self.sink.append(source);
    }
    pub fn stop(&self) {
        self.sink.stop();
    }
}
