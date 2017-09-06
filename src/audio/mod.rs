use rodio::{self, Sink, Source};
use std::fs::File;
use std::io::BufReader;

pub struct Sound {
    source: rodio::source::Buffered<rodio::Decoder<BufReader<File>>>,
}

impl Sound {
    pub fn new(path: &str) -> Self {
        let file = File::open(format!("media/{}", path)).unwrap();
        Sound {
            source: rodio::Decoder::new(BufReader::new(file))
                .unwrap()
                .buffered(),
        }
    }

    pub fn play(&self) {
        let endpoint = rodio::get_default_endpoint().unwrap();
        let sink = Sink::new(&endpoint);

        sink.append(self.source.clone());
        sink.detach();
    }
}

pub fn play(path: &str) {
    let endpoint = rodio::get_default_endpoint().unwrap();
    let file = File::open(format!("media/{}", path)).unwrap();

    let source = rodio::Decoder::new(BufReader::new(file)).unwrap();

    // source = ();
    // source.buffered() = ();
    // rodio::play_raw(&endpoint, source.convert_samples());
    // rodio::play_raw(&endpoint, source.convert_samples());
}

pub fn init() {}
