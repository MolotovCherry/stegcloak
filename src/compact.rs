use std::{
    io::{self, Write},
    string::FromUtf8Error,
};

use flate2::{
    write::{DeflateDecoder, DeflateEncoder},
    Compression,
};

pub fn compress(data: &str) -> Result<Vec<u8>, DeCompressError> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(data.as_bytes())?;

    let data = encoder.finish()?;

    Ok(data)
}

pub fn decompress(data: &[u8]) -> Result<String, DeCompressError> {
    let mut decoder = DeflateDecoder::new(Vec::new());

    decoder.write_all(data)?;
    let data = decoder.finish()?;

    Ok(String::from_utf8(data)?)
}

#[derive(Debug, thiserror::Error)]
pub enum DeCompressError {
    #[error("Failed to decode string: {0}")]
    StringError(#[from] FromUtf8Error),
    #[error("IoError: {0:?}")]
    IoError(#[from] io::Error),
}
