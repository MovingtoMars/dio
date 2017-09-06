use std::fs::OpenOptions;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::sync::mpsc;
use std::thread;

use rustc_serialize::json;

#[derive(Default, Clone, Copy, RustcDecodable, RustcEncodable)]
pub struct Stats {
    pub num_time_stops: u64,
    pub num_clicks: u64,
    pub num_key_presses: u64,
    pub num_startups: u64,
    pub num_knives_spawned: u64,
    pub total_game_time: f64,
}

enum Message {
    Save(Stats),
    Finish,
}

pub struct Handler {
    thread_handle: thread::JoinHandle<()>,
    latest_stats: Stats,
    sender: mpsc::Sender<Message>,
    send_counter: usize,
}

const FILENAME: &'static str = "stats.1.json";

impl Handler {
    /// This function may take a while to return, as it loads the stats file from disk (or creates a new one)
    /// It then spawns a child thread to handle saving updated stats asynchronously via the set() method.
    /// Call finish() to join the thread.
    pub fn new() -> Handler {
        let file = OpenOptions::new().read(true).write(true).open(FILENAME);

        let mut stats = Stats::default();

        let mut file = match file {
            Ok(mut file) => {
                let mut text = String::new();
                file.read_to_string(&mut text).unwrap();
                stats = json::decode(&text).unwrap();

                file
            }
            Err(err) => if err.kind() == ErrorKind::NotFound {
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(FILENAME)
                    .unwrap();

                let encoded = json::encode(&stats).unwrap();
                file.write_all(encoded.as_ref()).unwrap();
                file
            } else {
                panic!("Error loading {}: {}", FILENAME, err)
            },
        };

        let (sender, receiver) = mpsc::channel::<Message>();

        let child = thread::spawn(move || {
            loop {
                let stats = match receiver.recv() {
                    Ok(stats) => match stats {
                        Message::Save(stats) => stats,
                        Message::Finish => break,
                    },
                    Err(_) => break,
                };

                file.seek(SeekFrom::Start(0)).unwrap();
                let encoded = json::encode(&stats).unwrap();
                file.write_all(encoded.as_ref()).unwrap();
                file.set_len(encoded.len() as u64).unwrap();
            }

            file.flush().unwrap();
        });

        Handler {
            thread_handle: child,
            latest_stats: stats,
            sender: sender,
            send_counter: 0,
        }
    }

    pub fn get(&self) -> Stats {
        self.latest_stats
    }

    /// asynchronous
    pub fn set(&mut self, stats: Stats) {
        self.latest_stats = stats;
        self.send_counter += 1;
        if self.send_counter > 60 {
            self.send_counter = 0;
            self.sender.send(Message::Save(stats)).unwrap();
        }
    }

    /// returns once last save is completed
    pub fn finish(self) {
        self.sender.send(Message::Save(self.latest_stats)).unwrap();
        self.sender.send(Message::Finish).unwrap();
        self.thread_handle.join().unwrap();
    }
}
