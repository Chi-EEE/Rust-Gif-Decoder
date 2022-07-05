mod Gif;
use Gif::Decoder;

fn main() -> std::io::Result<()> {
    let mut gif_decoder = Decoder{ offset: 0 };
    gif_decoder.decode("./gifs/aware.gif");
    Ok(())
}
