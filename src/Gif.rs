use byteorder::{ByteOrder, BigEndian, LittleEndian};
use std::fmt; // 1.3.4

///

pub struct Gif {
    version: String,
}
impl Gif {
    // fn example(&self) {}
}

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
///
pub struct Decoder;
impl Decoder {
    pub fn decode(file_path: &str) -> Result<Gif, GifError> {
        let contents = std::fs::read(file_path).expect("Something went wrong reading the file");

        let mut contents = contents.as_slice();
        {
            let mut signature: String = String::new();
            match String::from_utf8(contents[0..=2].to_vec()) {
                Ok(parsed_signature) => {
                    signature = parsed_signature;
                }
                Err(err) => println!("{}", err),
            }
            if signature != "GIF" {
                return Err(GifError::SignatureError);
            }
        }
        let mut version: String = String::new();
        match String::from_utf8(contents[3..=5].to_vec()) {
            Ok(parsed_version) => {
                version = parsed_version;
            }
            Err(err) => println!("{}", err),
        }
        match version.as_str() {
            // as_str needs : _
            "89a" => {
                let width = LittleEndian::read_u16(&contents[6..=7]);
                let height = LittleEndian::read_u16(&contents[8..=9]);
                let packed_byte = LittleEndian::read_u16(&contents[10..=11]);
                let global_color_flag = (packed_byte & 0b1000_0000) != 0;
                let color_resolution = (packed_byte & 0b0111_0000) as u8;
                let sorted_flag = (packed_byte & 0b0000_1000) != 0;
                let global_color_size = (packed_byte & 0b0000_0111) as u8;

                let background_color_index = LittleEndian::read_u16(&contents[12..=13]);
                let pixel_aspect_ratio = LittleEndian::read_u16(&contents[14..=15]);

                let mut offset: usize = 16;
                let length: usize = 3 * usize::pow(2, (global_color_size + 1).into());

                // Global Color Table
                let mut i: usize = offset;
                let mut color_vector: Vec<Color> = Vec::new();

                while i < offset + length {
                    color_vector.push(Color{ red: contents[i], green: contents[i+1], blue: contents[i+2], alpha: 255 });
                    i = i + 3;
                }
                // End
                
                // Graphic Control Extension
                offset = offset + length;
                let extension_introducer = LittleEndian::read_u16(&contents[offset..offset+2]);
                if extension_introducer != 21 {
                    println!("Something went wrong here.")
                }
                offset = offset + 2;

                let graphic_control_label = LittleEndian::read_u16(&contents[offset..offset+2]);
                if graphic_control_label != 249 {
                    println!("Something went wrong here.")
                }
                offset = offset + 2;

                let byte_size = LittleEndian::read_u16(&contents[offset..offset+2]);
                offset = offset + 2;

                let packed_field = LittleEndian::read_u16(&contents[offset..offset+2]);
                let disposal_method = (packed_field & 0b0001_1100) as u8;
                let user_input_flag = (packed_field & 0b0000_0010) != 0;
                let transparent_color_flag = (packed_field & 0b0000_0001) != 0;
                offset = offset + 2;

                let delay_time = LittleEndian::read_u32(&contents[offset..offset+4]);
                offset = offset + 4;

                let transparent_color_index = LittleEndian::read_u16(&contents[offset..offset+2]);
                offset = offset + 2;
                
                let block_terminator = LittleEndian::read_u16(&contents[offset..offset+2]); // This must be "00"
                offset = offset + 2;
                // End
                
                
            }
            "87a" => {}
            _ => {}
        }
        return Ok(Gif { version: version });
    }
    // fn bark(&self) {}
}

///

#[derive(Debug)]
pub enum GifError {
    SignatureError,
    ConversionError,
}

impl std::error::Error for GifError {}

impl fmt::Display for GifError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GifError::SignatureError => write!(f, "Signature Error"),
            GifError::ConversionError => write!(f, "Conversion Error"),
        }
    }
}
