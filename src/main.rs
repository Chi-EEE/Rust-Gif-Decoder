mod Gif;

fn main() -> std::io::Result<()> {
    let mut gif_decoder = Gif::Decoder{ offset: 0 };
    gif_decoder.decode("./gifs/clap.gif");
    Ok(())
}
