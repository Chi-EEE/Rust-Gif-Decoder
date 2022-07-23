mod Gif;

fn main() -> std::io::Result<()> {
    match Gif::Decoder::decode("./gifs/clap.gif") {
        Ok(mut gif) => {
            let buffers = gif.process_frames();
            for buffer in buffers.into_iter() {
                println!("{:?}", buffer);
            }
        },
        Err(_) => {},
    };
    // let mut gif_decoder = Gif::Decoder::decode("./gifs/danger.gif");
    // let mut gif_decoder = Gif::Decoder::decode("./gifs/dj.gif"); // something wrong
    // let mut gif_decoder = Gif::Decoder::decode("./gifs/clap.gif");
    Ok(())
}
