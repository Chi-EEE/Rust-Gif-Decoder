mod Gif;

fn main() -> std::io::Result<()> {
    match Gif::Decoder::decode("./gifs/chest.gif") {
        Ok(mut gif) => {
            let frames = &gif.frames;
            // println!("Count: {}", frames.len());
            // for frame in frames.into_iter() {
            //     println!("{:?} {:?}", frame.im.width, frame.im.height);
            // }
            let buffers = &gif.process_frames();
            // for buffer in buffers.into_iter() {
            //     // println!("{:?}", buffer);
            //     println!("{}", buffer.len());
            // }
        },
        Err(_) => {},
    };
    // let mut gif_decoder = Gif::Decoder::decode("./gifs/danger.gif");
    // let mut gif_decoder = Gif::Decoder::decode("./gifs/dj.gif"); // something wrong
    // let mut gif_decoder = Gif::Decoder::decode("./gifs/clap.gif");
    Ok(())
}
