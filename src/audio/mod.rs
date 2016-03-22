extern crate sdl2;
extern crate sdl2_mixer;

use sdl2_mixer::{Chunk, Channel};

use std::path::Path;

use media;

use std::sync::Mutex;

lazy_static! {
    static ref CHANNELS: Mutex<Vec<ChannelID>> = Mutex::new(vec![]);
}

struct ChannelID {
    chan: Channel,
    used: bool,
}

pub fn init() {
    let init_len = 32;

    sdl2_mixer::allocate_channels(init_len);

    let mut vec = Vec::new();
    for i in 0..init_len {
        vec.push(ChannelID { chan: sdl2_mixer::channel(i), used: false })
    }

    vec[0].used = true;

    fn on_channel_finished(chan: Channel) {
        println!("{}", get_channel_id(chan));
        CHANNELS.lock().unwrap()[get_channel_id(chan) as usize].used = false;
    }

    sdl2_mixer::set_channel_finished(on_channel_finished);

    CHANNELS.lock().unwrap().append(&mut vec);
}

fn get_channel_id(mut chan: Channel) -> isize {
    struct Uchan(isize);
    unsafe {
        let chanptr = &mut chan as *mut Channel;
        let uchan = chanptr as *mut Uchan;
        let Uchan(val) = *uchan;
        val
    }
}

fn next_channel() -> Channel {
    for ref mut c in &mut CHANNELS.lock().unwrap().iter_mut() {
        if !c.used {
            c.used = true;
            return c.chan;
        }
    }

    panic!("oh no (TODO more channels)");
}

pub struct Sound {
    chunk: Chunk,
}

impl Sound {
    pub fn new(media_handle: &media::MediaHandle, path: &str) -> Sound {
        let mut full_path = media_handle.base_path.clone();
        full_path.push(&Path::new(path));

        Sound {
            chunk: Chunk::from_file(full_path.as_path()).unwrap(),
        }
    }

    pub fn play(&self, loops: isize) {
        let chan = next_channel();
        chan.play(&self.chunk, loops).unwrap();
        //chan.set_distance(200);
    }
}