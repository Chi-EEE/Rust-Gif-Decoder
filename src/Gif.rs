use byteorder::{ByteOrder, LittleEndian};
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
                let width = LittleEndian::read_u16(&contents[6..8]);
                let height = LittleEndian::read_u16(&contents[8..10]);
                let packed_field = contents[10];
                let global_color_flag = (packed_field & 0b1000_0000) != 0;
                let color_resolution = (packed_field & 0b0111_0000) as u8;
                let sorted_flag = (packed_field & 0b0000_1000) != 0;
                let global_color_size = (packed_field & 0b0000_0111) as u8;

                let background_color_index = contents[11];
                let pixel_aspect_ratio = contents[12];

                let mut offset: usize = 12;
                // Global Color Table
                let length: usize = 3 * usize::pow(2, (global_color_size + 1).into());
                let mut i: usize = offset;
                let mut global_color_vector: Vec<Color> = Vec::new();

                while i < offset + length {
                    global_color_vector.push(Color {
                        red: contents[i],
                        green: contents[i + 1],
                        blue: contents[i + 2],
                        alpha: 255,
                    });
                    i = i + 3;
                }
                offset = offset + length;
                // End

                // Graphic Control Extension
                let extension_introducer = contents[offset];
                if extension_introducer != 0x21 {
                    println!("Something went wrong here.");
                }
                offset = offset + 1;

                let graphic_control_label = contents[offset];
                if graphic_control_label != 0xF9 {
                    println!("Something went wrong here.");
                }
                offset = offset + 1;

                let byte_size = contents[offset];
                offset = offset + 1;

                let packed_field = contents[offset];
                let disposal_method = (packed_field & 0b0001_1100) as u8;
                let user_input_flag = (packed_field & 0b0000_0010) != 0;
                let transparent_color_flag = (packed_field & 0b0000_0001) != 0;
                offset = offset + 1;

                let delay_time = LittleEndian::read_u16(&contents[offset..offset + 2]);
                offset = offset + 2;

                let transparent_color_index = contents[offset];
                offset = offset + 1;

                let block_terminator = contents[offset]; // This must be 00
                offset = offset + 1;
                // End

                // Image Descriptor
                let image_separator =contents[offset]; // This must be "2C" or 44
                offset = offset + 1;

                let image_left = LittleEndian::read_u16(&contents[offset..offset + 2]);
                offset = offset + 2;

                let image_top = LittleEndian::read_u16(&contents[offset..offset + 2]);
                offset = offset + 2;

                let image_width = LittleEndian::read_u16(&contents[offset..offset + 2]);
                offset = offset + 2;

                let image_height = LittleEndian::read_u16(&contents[offset..offset + 2]);
                offset = offset + 2;

                let packed_field = contents[offset];
                let local_color_table_flag = (packed_field & 0b1000_0000) as u8;
                let interface_flag = (packed_field & 0b0100_0000) as u8;
                let sort_flag = (packed_field & 0b0010_0000) as u8;
                // let _ = (packed_field & 0b0001_1000) as u8; // Future use
                let local_color_table_size = (packed_field & 0b0000_0111) as u8;
                offset = offset + 1;
                // End

                // Local Color Table
                let length: usize = 3 * usize::pow(2, (local_color_table_size + 1).into());
                let mut i: usize = offset;
                let mut local_color_vector: Vec<Color> = Vec::new();

                while i < offset + length {
                    local_color_vector.push(Color {
                        red: contents[i],
                        green: contents[i + 1],
                        blue: contents[i + 2],
                        alpha: 255,
                    });
                    i = i + 3;
                }
                offset = offset + length;
                // End

                // Image Data
                let LZW_minimum_code_size = contents[offset];
                offset = offset + 1;

                let mut data_sub_blocks_count = contents[offset];
                offset = offset + 1;
                loop {
                    for n in 0..data_sub_blocks_count {
                        let data_sub_block = contents[offset];
                        offset = offset + 1;
                    }
                    if data_sub_blocks_count <= 0 {
                        break;
                    }
                    data_sub_blocks_count = contents[offset];
                    offset = offset + 1;
                }
                // End

                // Plain Text Extension (Optional) / Application Extension (Optional) / Comment Extension (Optional) / Trailer
                let extension_introducer = contents[offset];
                offset = offset + 1;
                if extension_introducer == 0x3B {
                    println!("End of file.");
                    // End of file
                }
                let label = contents[offset];
                offset = offset + 1;
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
