use byteorder::{ByteOrder, LittleEndian};
use std::process::exit;
use std::{collections::HashMap, fmt}; // 1.3.4
mod DataHelper;
use DataHelper::BitReader;

///
#[derive(Default)]
pub struct Gif {
    version: String,
    lsd: LogicalScreenDescriptor,
    global_table: Option<Vec<Color>>,
    frames: Vec<ParsedFrame>,
}
impl Gif {
    // fn example(&mut self) {}
}

#[derive(Default)]
pub(crate) struct LogicalScreenDescriptor {
    width: u16,
    height: u16,
    global_color_flag: bool,
    color_resolution: u8,
    sorted_flag: bool,
    global_color_size: u8,
    background_color_index: u8,
    pixel_aspect_ratio: u8,
}

#[derive(Default)]
struct ParsedFrame {
    gcd: GraphicsControlExtension,
    im: ImageDescriptor,
}

#[derive(Default)]
pub(crate) struct ImageDescriptor {
    left: u16,
    top: u16,
    width: u16,
    height: u16,
    local_color_table_flag: bool,
    interface_flag: bool,
    sort_flag: bool,
    local_color_table_size: u16,
}

#[derive(Default)]
pub(crate) struct GraphicsControlExtension {
    disposal_method: u8,
    user_input_flag: bool,
    transparent_color_flag: bool,
    delay_time: u16,
    transparent_color_index: u8,
}

#[derive(Clone)]
enum CodeTable {
    Color(Vec<u16>),
    Empty,
    Clear,
    End,
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
    pub fn decode(&mut self, file_path: &str) -> Result<(), ()> {
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
                // return Err(GifError::SignatureError);
            }
        }

        let mut gif = Gif::default();
        let mut version: String = String::new();
        match String::from_utf8(contents[3..6].to_vec()) {
            Ok(parsed_version) => {
                version = parsed_version;
            }
            Err(err) => println!("Error 2: {}", err),
        }
        gif.version = version;

        self.handle_logical_screen_descriptor(&mut gif, contents);

        self.offset = 13;
        // Global Color Table
        let length: usize = 3 * 2 << gif.lsd.global_color_size;
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
            let introducer = contents[self.offset];
            if introducer != 0x21 && introducer == 0x3B {
                break;
            }
            self.increment_offset(1);
            println!("Offset: {}", self.offset);

            if introducer == 0x2C {
                // Image Descriptor
                self.handle_image_descriptor(&mut gif, contents);
                continue;
            }
            let label = contents[self.offset];
            self.increment_offset(1);
            match label {
                0xF9 => {
                    self.handle_graphic_control_extension(&mut gif, contents);
                },
                0x01 => {
                    self.handle_plain_text_extension(&mut gif, contents);
                },
                0xFF => {
                    self.handle_application_extension(&mut gif, contents);
                },
                0xFE => {
                    self.handle_comment_extension(&mut gif, contents);
                },
                _ => {}
            }
        }
        // Trailer
        println!("End of file.");
        return Ok(());
    }
    fn increment_offset(&mut self, amount: usize) {
        self.offset += amount;
    }
    fn shl_or(&mut self, val: u16, shift: usize, def: u16) -> u16 {
        [val << (shift & 15), def][((shift & !7) != 0) as usize]
    }
    fn shr_or(&mut self, val: u8, shift: usize, def: u8) -> u8 {
        [val >> (shift & 7), def][((shift & !7) != 0) as usize]
    }
    fn handle_logical_screen_descriptor(&mut self, gif: &mut Gif, contents: &[u8]) {
        // Logic Screen Descriptor
        #[cfg(debug_assertions)]
        println!("Logic Screen Descriptor Offset: {}", self.offset);

        gif.lsd.width = LittleEndian::read_u16(&contents[6..8]); // width
        gif.lsd.height = LittleEndian::read_u16(&contents[8..10]); // height

        let packed_field = contents[10];

        gif.lsd.global_color_flag = (packed_field & 0b1000_0000) != 0; // global_color_flag
        gif.lsd.color_resolution = (packed_field & 0b0111_0000) as u8; // color_resolution
        gif.lsd.sorted_flag = (packed_field & 0b0000_1000) != 0; // sorted_flag
        gif.lsd.global_color_size = (packed_field & 0b0000_0111) as u8; // global_color_size

        gif.lsd.background_color_index = contents[11]; // background_color_index
        gif.lsd.pixel_aspect_ratio = contents[12]; // pixel_aspect_ratio
    }
    fn handle_graphic_control_extension(&mut self, gif: &mut Gif, contents: &[u8]) {
        // Graphical Control Extension
        #[cfg(debug_assertions)]
        println!("Graphic Control Extension Offset: {}", self.offset);

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
        println!("{}", transparent_color_index);
        self.increment_offset(1);

        let block_terminator = contents[self.offset]; // This must be 00
        self.increment_offset(1);
        // End
    }
    fn handle_image_descriptor(&mut self, gif: &mut Gif, contents: &[u8]) {
        // Image Descriptor
        #[cfg(debug_assertions)]
        println!("Image Descriptor Offset: {}", self.offset);

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
        let interface_flag = (packed_field & 0b0100_0000) != 0;
        let sort_flag = (packed_field & 0b0010_0000) != 0;
        // let _ = (packed_field & 0b0001_1000) as u8; // Future use
        let local_color_table_size = (packed_field & 0b0000_0111) as u8;
        self.increment_offset(1);
        // End

        // Local Color Table
        if (local_color_table_flag) {
            let length: usize = 3 * 2 << local_color_table_size;
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
            println!(
                "End of local color table: {}, Length: {}",
                self.offset, length
            );
        }
        // End

        // Image Data
        #[cfg(debug_assertions)]
        println!("Image Data Offset: {}", self.offset);

        let lzw_minimum_code_size = contents[self.offset];
        self.increment_offset(1);

        // Data sub block section
        let mut data_sub_blocks_count = contents[self.offset];
        self.increment_offset(1);

        let mut index_stream: Vec<u8> = Vec::new();

        let mut code_table: HashMap<usize, Vec<u8>> = HashMap::new();
        let mut code_stream: Vec<u16> = Vec::new();

        let clear_code = self.shl_or(2, (lzw_minimum_code_size - 1).into(), 0);
        let eoi_code = clear_code + 1;

        let mut last_code = eoi_code;
        let mut size: usize = (lzw_minimum_code_size + 1).into();
        let mut grow_code = clear_code - 1;

        let mut is_initalized = false;

        let mut br = BitReader::new();
        loop {
            let offset_add: usize = self.offset + data_sub_blocks_count as usize;
            let sliced_bytes = &contents[self.offset..offset_add];

            br.push_bytes(&sliced_bytes);
            loop {
                let code = match br.read_bits(size) {
                    Ok(bits) => bits,
                    Err(err) => {
                        println!("{}", err);
                        exit(0x0);
                    }
                };
                if code == eoi_code {
                    code_stream.push(code);
                    break;
                } else if code == clear_code {
                    code_stream = Vec::new();
                    code_table = HashMap::new();
                    for n in 0..eoi_code {
                        if n < clear_code {
                            code_table.insert(n as usize, vec![n as u8]);
                        } else {
                            code_table.insert(n as usize, vec![]);
                        }
                    }
                    last_code = eoi_code;
                    size = (lzw_minimum_code_size + 1).into();
                    grow_code = (2 << size - 1) - 1;
                    is_initalized = false;
                } else if !is_initalized {
                    match code_table.get(&(code as usize)) {
                        Some(codes) => {
                            index_stream.extend(codes);
                        },
                        None => {
                            println!("invalid code");
                            exit(1);
                        },
                    }
                    is_initalized = true;
                } else {
                    let mut k: u8 = 0;
                    let prev_code = code_stream[code_stream.len() - 1];
                    if code <= last_code {
                        match code_table.get(&(code as usize)) {
                            Some(codes) => {
                                index_stream.extend(codes);
                                k = codes[0];
                            },
                            None => {
                                println!("invalid code");
                                exit(2);
                            },
                        }
                    } else {
                        match code_table.get(&(prev_code as usize)) {
                            Some(codes) => {
                                k = codes[0];
                                index_stream.extend(codes);
                                index_stream.push(k);
                            },
                            None => {
                                println!("invalid code");
                                exit(3);
                            },
                        }
                    }
                    if last_code < 0xFFF {
                        match code_table.get(&(prev_code as usize)) {
                            Some(codes) => {
                                last_code += 1;
                                let mut last_code_table = codes.to_vec();
                                last_code_table.push(k);
                                code_table.insert(last_code as usize, last_code_table);
                                if last_code == grow_code && last_code < 0xFFF {
                                    size += 1;
                                    grow_code = (2 << size - 1) - 1;
                                }
                            },
                            None => {
                                println!("invalid code");
                                exit(4);
                            },
                        }
                    }
                }
                code_stream.push(code);
                let has_bits = match br.has_bits(size) {
                    Ok(has_bits) => has_bits,
                    Err(err) => {
                        println!("{}", err);
                        exit(0x0);
                    }
                };
                if !has_bits {
                    break;
                }
            }

            self.offset = offset_add;
            data_sub_blocks_count = contents[self.offset];
            self.increment_offset(1);
            if data_sub_blocks_count == 0 {
                break;
            }
        }
    }
    fn handle_plain_text_extension(&mut self, gif: &mut Gif, contents: &[u8]) {
        // Plain Text Extension (Optional)
        #[cfg(debug_assertions)]
        println!("Plain Text Extension Offset: {}", self.offset);

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
            if data_sub_blocks_count == 0x00 {
                break;
            }
        }
    }
    fn handle_application_extension(&mut self, gif: &mut Gif, contents: &[u8]) {
        // Application Extension (Optional)
        #[cfg(debug_assertions)]
        println!("Application Extension Offset: {}", self.offset);

        let block_size: usize = contents[self.offset].into();
        self.increment_offset(1);

        let mut application = String::from("");
        let length = self.offset + block_size;
        match String::from_utf8(contents[self.offset..length].to_vec()) {
            Ok(parsed_application) => {
                application = parsed_application;
            }
            Err(err) => println!("Error 3: {}", err),
        }
        self.increment_offset(block_size);

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
    fn handle_comment_extension(&mut self, gif: &mut Gif, contents: &[u8]) {
        // Comment Extension (Optional)
        #[cfg(debug_assertions)]
        println!("Comment Extension Offset: {}", self.offset);

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

pub struct ByteSizeError {
    len: usize,
    rbits: Option<usize>,
    file: &'static str,
    line: u32,
    column: u32,
}

impl fmt::Display for ByteSizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let err_msg = match self.rbits {
            Some(rbits) => format!(
                "Not enough bytes to read {} bits (read {} bits) --> {}:{}:{}",
                self.len,
                rbits,
                self.file,
                self.line,
                self.column
            ),
            None => format!(
                "Exceeds max bit size: ${} (max: 12) --> {}:{}:{}",
                self.len,
                self.file,
                self.line,
                self.column
            ),
        };
        write!(f, "{}", err_msg)
    }
}