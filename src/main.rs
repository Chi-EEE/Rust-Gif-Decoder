mod Gif;
use Gif::Decoder;

use std::convert::TryFrom;
use std::{fs, str::Utf8Error};


fn main() -> std::io::Result<()> {
    Decoder::decode("./gifs/aware.gif");
    Ok(())
}
