use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::ops::IndexMut;

const MAX_STACK_SIZE: u16 = 4096;

fn shl_or(val: u32, shift: usize, def: u32) -> u32 {
    [val << (shift & 31), def][((shift & !31) != 0) as usize]
}
fn shr_or(val: u32, shift: usize, def: u32) -> u32 {
    [val >> (shift & 31), def][((shift & !31) != 0) as usize]
}

#[derive(Default)]
pub struct Gif {
    pub version: String,
    pub lsd: LogicalScreenDescriptor,
    pub global_table: Vec<Color>,
    pub frames: Vec<Frame>,
}
impl Gif {
    pub fn process_frames(&mut self) -> Vec<Vec<u8>> {
        let mut buffers: Vec<Vec<u8>> = Vec::new();
        let frames_iter = self.frames.iter();
        for frame in frames_iter {
            let mut buffer: Vec<u8> = Vec::new();
            if (frame.im.local_color_table_flag) {
                for index in (&frame.index_stream).into_iter() {
                    let color = frame.local_table.get(*index as usize).unwrap();
                    buffer.push(color.red);
                    buffer.push(color.green);
                    buffer.push(color.blue);
                    if frame.gcd.transparent_color_flag
                        && index == (&frame.gcd.transparent_color_index)
                    {
                        buffer.push(0);
                    } else {
                        buffer.push(255);
                    }
                }
            } else {
                for index in (&frame.index_stream).into_iter() {
                    let color = self.global_table.get(*index as usize).unwrap();
                    buffer.push(color.red);
                    buffer.push(color.green);
                    buffer.push(color.blue);
                    if frame.gcd.transparent_color_flag
                        && index == (&frame.gcd.transparent_color_index)
                    {
                        buffer.push(0);
                    } else {
                        buffer.push(255);
                    }
                }
            }
            buffers.push(buffer);
        }
        return buffers;
    }
}

#[derive(Default)]
pub struct LogicalScreenDescriptor {
    pub width: u16,
    pub height: u16,
    pub global_color_flag: bool,
    pub color_resolution: u8,
    pub sorted_flag: bool,
    pub global_color_size: u8,
    pub background_color_index: u8,
    pub pixel_aspect_ratio: u8,
}
#[derive(Default)]
pub struct Frame {
    pub gcd: GraphicsControlExtension,
    pub im: ImageDescriptor,
    pub local_table: Vec<Color>,
    pub(crate) index_stream: Vec<u8>,
}

#[derive(Default)]
pub struct ImageDescriptor {
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
    pub local_color_table_flag: bool,
    pub interface_flag: bool,
    pub sort_flag: bool,
    pub local_color_table_size: u8,
}

#[derive(Default)]
pub struct GraphicsControlExtension {
    pub disposal_method: u8,
    pub user_input_flag: bool,
    pub transparent_color_flag: bool,
    pub delay_time: u16,
    pub transparent_color_index: u8,
}

pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}
///
pub(crate) struct Decoder {}

impl Decoder {
    pub fn decode(file_path: &str) -> Result<Gif, ()> {
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
                return Err(());
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

        Self::handle_logical_screen_descriptor(&mut gif, contents);

        let mut offset: usize = 13;

        // Global Color Table
        let length: usize = 3 * 2 << gif.lsd.global_color_size;
        let mut i: usize = offset;

        if gif.lsd.global_color_flag {
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
            Self::increment_offset(&mut offset, length);
            gif.global_table = global_color_vector;
        }
        // End
        let mut done = false;
        loop {
            let introducer = contents[offset];
            Self::increment_offset(&mut offset, 1);
            match introducer {
                0x2C => {
                    // Image Descriptor
                    Self::handle_image_descriptor(&mut offset, &mut gif, contents);
                }
                0x21 => {
                    let label = contents[offset];
                    Self::increment_offset(&mut offset, 1);
                    match label {
                        0xF9 => {
                            Self::handle_graphic_control_extension(&mut offset, &mut gif, contents);
                        }
                        0x01 => {
                            Self::handle_plain_text_extension(&mut offset, &mut gif, contents);
                        }
                        0xFF => {
                            Self::handle_application_extension(&mut offset, &mut gif, contents);
                        }
                        0xFE => {
                            Self::handle_comment_extension(&mut offset, &mut gif, contents);
                        }
                        _ => {}
                    }
                }
                0x3B => {
                    done = true;
                }
                0x00 => {}
                _ => {}
            }
            if done {
                break;
            }
        }
        // Trailer
        println!("End of file.");
        return Ok(gif);
    }
    fn skip(offset: &mut usize, contents: &[u8]) {
        loop {
            let data_sub_blocks_count = contents[*offset];
            Self::increment_offset(offset, 1);
            if data_sub_blocks_count > 0 {
                Self::increment_offset(offset, data_sub_blocks_count.into());
            } else {
                break;
            }
            if *offset >= contents.len() - 1 {
                break;
            }
        }
    }
    fn increment_offset(offset: &mut usize, amount: usize) {
        *offset += amount;
    }
    fn handle_logical_screen_descriptor(gif: &mut Gif, contents: &[u8]) {
        // Logic Screen Descriptor
        #[cfg(debug_assertions)]
        println!("Logic Screen Descriptor Offset: {}", 6);

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
    fn handle_graphic_control_extension(offset: &mut usize, gif: &mut Gif, contents: &[u8]) {
        // Graphical Control Extension
        #[cfg(debug_assertions)]
        println!("Graphic Control Extension Offset: {}", *offset);

        let mut parsed_frame: Frame = Frame::default();

        let byte_size = contents[*offset];
        Self::increment_offset(offset, 1);

        let packed_field = contents[*offset];
        parsed_frame.gcd.disposal_method = (packed_field & 0b0001_1100) as u8;
        parsed_frame.gcd.user_input_flag = (packed_field & 0b0000_0010) != 0;
        parsed_frame.gcd.transparent_color_flag = (packed_field & 0b0000_0001) != 0;
        Self::increment_offset(offset, 1);

        parsed_frame.gcd.delay_time = LittleEndian::read_u16(&contents[*offset..*offset + 2]);
        Self::increment_offset(offset, 2);

        parsed_frame.gcd.transparent_color_index = contents[*offset];
        Self::increment_offset(offset, 1);

        let block_terminator = contents[*offset]; // This must be 00 ///////////////////////////////////////////////////////////////////
        Self::increment_offset(offset, 1);
        // End

        gif.frames.push(parsed_frame);
    }
    fn handle_image_descriptor(offset: &mut usize, gif: &mut Gif, contents: &[u8]) {
        // Image Descriptor
        #[cfg(debug_assertions)]
        println!("Image Descriptor Offset: {}", *offset);

        let frame_index = gif.frames.len() - 1;
        let mut parsed_frame = &mut gif.frames[frame_index];

        parsed_frame.im.left = LittleEndian::read_u16(&contents[*offset..*offset + 2]); // image_left
        Self::increment_offset(offset, 2);

        parsed_frame.im.top = LittleEndian::read_u16(&contents[*offset..*offset + 2]); // image_top
        Self::increment_offset(offset, 2);

        parsed_frame.im.width = LittleEndian::read_u16(&contents[*offset..*offset + 2]); // image_width
        Self::increment_offset(offset, 2);

        parsed_frame.im.height = LittleEndian::read_u16(&contents[*offset..*offset + 2]); // image_height
        Self::increment_offset(offset, 2);

        let packed_field = contents[*offset];
        parsed_frame.im.local_color_table_flag = (packed_field & 0b1000_0000) != 0;
        parsed_frame.im.interface_flag = (packed_field & 0b0100_0000) != 0;
        parsed_frame.im.sort_flag = (packed_field & 0b0010_0000) != 0;
        // let _ = (packed_field & 0b0001_1000) as u8; // Future use
        parsed_frame.im.local_color_table_size = (packed_field & 0b0000_0111) as u8;
        Self::increment_offset(offset, 1);
        // End

        // Local Color Table
        if parsed_frame.im.local_color_table_flag {
            let length: usize = 3 * 2 << parsed_frame.im.local_color_table_size;
            let mut i: usize = *offset;
            let mut local_color_vector: Vec<Color> = Vec::new();

            while i < *offset + length {
                local_color_vector.push(Color {
                    red: contents[i],
                    green: contents[i + 1],
                    blue: contents[i + 2],
                    alpha: 255,
                });
                i = i + 3;
            }
            Self::increment_offset(offset, length);
            println!("End of local color table: {}, Length: {}", *offset, length);
            parsed_frame.local_table = local_color_vector;
        }
        let null_code: i32 = -1;
        let npix = gif.lsd.width * gif.lsd.height;

        // Initialize GIF data stream decoder.
        let lzw_minimum_code_size = contents[*offset];
        Self::increment_offset(offset, 1);

        let clear_code = shl_or(1, lzw_minimum_code_size as usize, 0);
        let eoi_code = clear_code + 1;
        let mut available = clear_code + 2;
        let mut old_code = null_code;
        let mut code_size: usize = (lzw_minimum_code_size + 1) as usize;
        println!("{}, {}, {}", *offset, lzw_minimum_code_size, code_size);
        let mut code_mask = shl_or(1, code_size, 0) - 1;

        let mut prefix: Vec<u16> = vec![0; MAX_STACK_SIZE as usize]; // No need to fill with 0 (already filled)
        let mut suffix: Vec<u8> = vec![0; MAX_STACK_SIZE as usize];
        for code in 0..clear_code {
            *suffix.index_mut(code as usize) = code as u8;
        }

        let mut pixel_stack: Vec<u8> = vec![0; (MAX_STACK_SIZE + 1) as usize];
        let mut top = 0;

        let mut index_stream: Vec<u8> = Vec::new();

        let mut block: &[u8] = &[0];

        let mut in_code = 0;
        let mut first: u8 = 0;
        let mut datum: u32 = 0;
        let mut bits = 0;
        let mut data_sub_blocks_count = 0;
        let mut bi = 0;

        let mut n = 0;
        while n < npix {
            if top == 0 {
                if bits < code_size {
                    if data_sub_blocks_count == 0 {
                        data_sub_blocks_count = contents[*offset];
                        Self::increment_offset(offset, 1);
                        if data_sub_blocks_count == 0 {
                            break;
                        }
                        let offset_add: usize = *offset + data_sub_blocks_count as usize;
                        block = &contents[*offset..offset_add];
                        *offset = offset_add;
                        
                        bi = 0;
                    }
                    datum += shl_or(block[bi as usize] as u32 & 0xFF, bits, 0);
                    bits += 8;
                    bi += 1;
                    data_sub_blocks_count -= 1;
                    continue;
                }
                let mut code = datum & code_mask;
                datum = shr_or(datum, code_size, 0);
                bits -= code_size;
                println!("{} {} {} {} {} {}", code_mask, n, code, available, eoi_code, datum);
                if code > available || code == eoi_code {
                    break;
                }
                if code == clear_code {
                    code_size = (lzw_minimum_code_size + 1) as usize;
                    code_mask = shl_or(1, code_size, 0) - 1;
                    available = clear_code + 2;
                    old_code = null_code;
                    continue;
                }
                if old_code == null_code {
                    *pixel_stack.index_mut(top as usize) = suffix[code as usize];
                    top += 1;
                    old_code = code as i32;
                    first = code as u8;
                    continue;
                }
                in_code = code;
                if code == available {
                    *pixel_stack.index_mut(top as usize) = first as u8;
                    top += 1;
                    code = old_code as u32;
                }
                while code > clear_code {
                    *pixel_stack.index_mut(top as usize) = suffix[code as usize];
                    top += 1;
                    code = prefix[code as usize] as u32;
                }
                first = suffix[code as usize] & 0xFF;

                *pixel_stack.index_mut(top as usize) = first;
                top += 1;

                if available < MAX_STACK_SIZE as u32 {
                    *prefix.index_mut(available as usize) = old_code as u16;
                    *suffix.index_mut(available as usize) = first;
                    available += 1;
                    if (available & code_mask) == 0 && available < MAX_STACK_SIZE as u32 {
                        code_size += 1;
                        code_mask += available;
                    }
                }
                old_code = in_code as i32;
            }
            top -= 1;
            index_stream.push(pixel_stack[top]);
            n += 1;
        }
        for _ in index_stream.len()..npix as usize {
            index_stream.push(0);// clear missing pixels
        }
        // End
        parsed_frame.index_stream = index_stream;
    }
    fn handle_plain_text_extension(offset: &mut usize, gif: &mut Gif, contents: &[u8]) {
        // Plain Text Extension (Optional)
        #[cfg(debug_assertions)]
        println!("Plain Text Extension Offset: {}", *offset);

        let block_size: usize = contents[*offset].into();
        Self::increment_offset(offset, 1 + block_size);

        Self::skip(offset, contents);
    }
    fn handle_application_extension(offset: &mut usize, gif: &mut Gif, contents: &[u8]) {
        // Application Extension (Optional)
        #[cfg(debug_assertions)]
        println!("Application Extension Offset: {}", *offset);

        let block_size: usize = contents[*offset].into();
        Self::increment_offset(offset, 1);

        let mut application = String::from("");
        let length = *offset + block_size;
        match String::from_utf8(contents[*offset..length].to_vec()) {
            Ok(parsed_application) => {
                application = parsed_application;
            }
            Err(err) => println!("Attempt to get application failed: {}", err),
        }
        Self::increment_offset(offset, block_size);

        Self::skip(offset, contents);
    }
    fn handle_comment_extension(offset: &mut usize, gif: &mut Gif, contents: &[u8]) {
        // Comment Extension (Optional)
        #[cfg(debug_assertions)]
        println!("Comment Extension Offset: {}", *offset);

        Self::skip(offset, contents);
    }
}
