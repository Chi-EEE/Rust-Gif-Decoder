use byteorder::{ByteOrder, LittleEndian};
use std::{fmt}; // 1.3.4

///

pub struct Gif {
    version: String,
}
impl Gif {
    // fn example(&mut self) {}
}

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
///
pub struct Decoder {
    pub offset: usize,
}

impl Decoder {
    pub fn decode(&mut self, file_path: &str) -> Result<Gif, GifError> {
        let contents = std::fs::read(file_path).expect("Something went wrong reading the file");

        let mut contents = contents.as_slice();
        {
            let mut signature: String = String::new();
            match String::from_utf8(contents[0..3].to_vec()) {
                Ok(parsed_signature) => {
                    signature = parsed_signature;
                }
                Err(err) => println!("Error 1: {}", err),
            }
            if signature != "GIF" {
                return Err(GifError::SignatureError);
            }
        }
        let mut version: String = String::new();
        match String::from_utf8(contents[3..6].to_vec()) {
            Ok(parsed_version) => {
                version = parsed_version;
            }
            Err(err) => println!("Error 2: {}", err),
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

                self.offset = 13;
                // Global Color Table
                let length: usize = 3 * usize::pow(2, (global_color_size + 1).into());
                let mut i: usize = self.offset;
                let mut global_color_vector: Vec<Color> = Vec::new();

                while i < self.offset + length {
                    global_color_vector.push(Color {
                        red: contents[i],
                        green: contents[i + 1],
                        blue: contents[i + 2],
                        alpha: 255,
                    });
                    i = i + 3;
                }
                self.increment_offset(length);
                // End
                loop {
                    let extension_introducer = contents[self.offset];
                    if extension_introducer != 0x21 && extension_introducer == 0x3B {
                        break;
                    }
                    println!("Offset: {}", self.offset);
                    self.increment_offset(1);

                    let label = contents[self.offset];
                    self.increment_offset(1);
                    match label {
                        0xF9 => {
                            self.handle_graphic_control_extension(contents);
                        }
                        0x01 => {
                            self.handle_plain_text_extension(contents);
                        }
                        0xFF => {
                            self.handle_application_extension(contents);
                        }
                        0xFE => {
                            self.handle_comment_extension(contents);
                        }
                        _ => {}
                    }
                }
                // Trailer
                println!("End of file.");
            }
            "87a" => {}
            _ => {}
        }
        return Ok(Gif { version: version });
    }
    fn increment_offset(&mut self, amount: usize) {
        self.offset += amount;
    }
    fn handle_graphic_control_extension(&mut self, contents: &[u8]) {
        // Graphical Control Extension
        let byte_size = contents[self.offset];
        self.increment_offset(1);

        let packed_field = contents[self.offset];
        let disposal_method = (packed_field & 0b0001_1100) as u8;
        let user_input_flag = (packed_field & 0b0000_0010) != 0;
        let transparent_color_flag = (packed_field & 0b0000_0001) != 0;
        self.increment_offset(1);

        let delay_time = LittleEndian::read_u16(&contents[self.offset..self.offset + 2]);
        self.increment_offset(2);

        let transparent_color_index = contents[self.offset];
        self.increment_offset(1);

        let block_terminator = contents[self.offset]; // This must be 00
        self.increment_offset(1);
        // End

        // Image Descriptor
        let image_separator = contents[self.offset]; // This must be "2C" or 44
        self.increment_offset(1);

        let image_left = LittleEndian::read_u16(&contents[self.offset..self.offset + 2]);
        self.increment_offset(2);

        let image_top = LittleEndian::read_u16(&contents[self.offset..self.offset + 2]);
        self.increment_offset(2);

        let image_width = LittleEndian::read_u16(&contents[self.offset..self.offset + 2]);
        self.increment_offset(2);

        let image_height = LittleEndian::read_u16(&contents[self.offset..self.offset + 2]);
        self.increment_offset(2);

        let packed_field = contents[self.offset];
        let local_color_table_flag = (packed_field & 0b1000_0000) != 0;
        let interface_flag = (packed_field & 0b0100_0000)  != 0;
        let sort_flag = (packed_field & 0b0010_0000) != 0;
        // let _ = (packed_field & 0b0001_1000) as u8; // Future use
        let local_color_table_size = (packed_field & 0b0000_0111) as u8;
        self.increment_offset(1);
        // End
        println!("Image Offset: {}", self.offset);

        // Local Color Table
        if (local_color_table_flag) {
            let length: usize = 3 * usize::pow(2, (local_color_table_size + 1).into());
            let mut i: usize = self.offset;
            let mut local_color_vector: Vec<Color> = Vec::new();
    
            while i < self.offset + length {
                local_color_vector.push(Color {
                    red: contents[i],
                    green: contents[i + 1],
                    blue: contents[i + 2],
                    alpha: 255,
                });
                i = i + 3;
            }
            self.increment_offset(length);
        }
        // End

        // Image Data
        let lzw_minimum_code_size = contents[self.offset];
        self.increment_offset(1);

        // Data sub block section
        let mut data_sub_blocks_count = contents[self.offset];
        self.increment_offset(1);
        loop {
            for n in 0..data_sub_blocks_count {
                let data_sub_block = contents[self.offset];
                self.increment_offset(1);
            }
            data_sub_blocks_count = contents[self.offset];
            self.increment_offset(1);
            if data_sub_blocks_count == 0 {
                break;
            }
        }
    }
    fn handle_plain_text_extension(&mut self, contents: &[u8]) {
        // Plain Text Extension (Optional)
        let block_size: usize = contents[self.offset].into();
        self.increment_offset(1 + block_size);

        // Data sub block section
        let mut data_sub_blocks_count = contents[self.offset];
        self.increment_offset(1);
        loop {
            let mut data_sub_block;
            for n in 0..data_sub_blocks_count {
                data_sub_block = contents[self.offset];
                self.increment_offset(1);
            }
            data_sub_blocks_count = contents[self.offset];
            self.increment_offset(1);
            println!("count: {}", data_sub_blocks_count);
            if data_sub_blocks_count == 0x00 {
                break;
            }
        }
    }
    fn handle_application_extension(&mut self, contents: &[u8]) {
        // Application Extension (Optional)
        let mut total_length: usize = 0;
        let block_size: usize = contents[self.offset].into();
        self.increment_offset(1);
        total_length += 1;

        let mut application = String::from("");
        let length = self.offset + block_size;
        match String::from_utf8(contents[self.offset..length].to_vec()) {
            Ok(parsed_application) => {
                application = parsed_application;
            }
            Err(err) => println!("Error 3: {}", err),
        }
        self.increment_offset(block_size);
        total_length += block_size;

        println!("Application: {:?}", application);
        // Data sub block section
        let mut data_sub_blocks_count = contents[self.offset];
        self.increment_offset(1);
        total_length += 1;
        loop {
            for n in 0..data_sub_blocks_count {
                let data_sub_block = contents[self.offset];
                self.increment_offset(1);
                total_length += 1;
            }
            data_sub_blocks_count = contents[self.offset];
            self.increment_offset(1);
            total_length += 1;
            if data_sub_blocks_count == 0 {
                break;
            }
        }
        println!("Total Length: {}", total_length);
    }
    fn handle_comment_extension(&mut self, contents: &[u8]) {
        // Comment Extension (Optional)
        let mut data_sub_blocks_count = contents[self.offset];
        self.increment_offset(1);
        loop {
            for n in 0..data_sub_blocks_count {
                let data_sub_block = contents[self.offset];
                self.increment_offset(1);
            }
            data_sub_blocks_count = contents[self.offset];
            self.increment_offset(1);
            if data_sub_blocks_count == 0 {
                break;
            }
        }
    }
}

///

#[derive(Debug)]
pub enum GifError {
    SignatureError,
}

impl std::error::Error for GifError {}

impl fmt::Display for GifError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GifError::SignatureError => write!(f, "Signature Error"),
        }
    }
}
