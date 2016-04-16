use std::fs::OpenOptions;
use std::io::{self, Read};
use std::path::Path;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

use rustc_serialize::json;

use media;

#[derive(Debug)]
pub enum LevelError {
    IoError(io::Error),
    DecoderError(json::DecoderError),
}

impl Display for LevelError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        fmt::Debug::fmt(&self, f)
    }
}

impl StdError for LevelError {
    fn description(&self) -> &str {
        "level error"
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            LevelError::DecoderError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<json::DecoderError> for LevelError {
    fn from(err: json::DecoderError) -> LevelError {
        LevelError::DecoderError(From::from(err))
    }
}

impl From<io::Error> for LevelError {
    fn from(err: io::Error) -> LevelError {
        LevelError::IoError(From::from(err))
    }
}

#[derive(Default, Clone, RustcDecodable, RustcEncodable)]
pub struct Level {
    pub name: String,
}

impl Level {
    pub fn new(media_handle: &media::MediaHandle, path: &str) -> Result<Level, LevelError> {
        let mut full_path = media_handle.base_path.clone();
        full_path.push(&Path::new(path));


        let mut file = try!(OpenOptions::new()
                       .read(true)
                       .open(full_path));

        let mut text = String::new();
        try!(file.read_to_string(&mut text));
        Ok(try!(json::decode(&text)))
    }
}
