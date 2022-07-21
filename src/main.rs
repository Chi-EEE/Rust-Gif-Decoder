mod Gif;

fn main() -> std::io::Result<()> {
    match Gif::Decoder::decode("./gifs/shake.gif") {
        Ok(gif) => {
            println!("{:?}", gif.version);
        },
        Err(_) => {},
    };
    // let mut gif_decoder = Gif::Decoder::decode("./gifs/danger.gif");
    // let mut gif_decoder = Gif::Decoder::decode("./gifs/dj.gif"); // something wrong
    // let mut gif_decoder = Gif::Decoder::decode("./gifs/clap.gif");
    Ok(())
}
