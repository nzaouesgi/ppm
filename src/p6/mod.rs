use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, Error, ErrorKind, SeekFrom};
use std::path::Path;

extern crate test;
extern crate num_cpus;

// Here we use u32 because we still need to implement support for > 255 rgb_max_value files.
pub struct BinaryPixel {
    r: u32,
    g: u32,
    b: u32,
}

// Invert a pixel's values.
pub fn invert_binary_pixel(pixel: &mut BinaryPixel) -> () {
    pixel.r = 255 - pixel.r;
    pixel.b = 255 - pixel.b;
    pixel.g = 255 - pixel.g;
}

// Turn a pixel into greyscale.
pub fn greyscale_binary_pixel(pixel: &mut BinaryPixel) -> () {
    let grey = (pixel.r + pixel.b + pixel.g) / 3 ;
    pixel.r = grey;
    pixel.b = grey;
    pixel.g = grey;
}

/* 
    Our BinaryImage structure.
    The only data which is stored inside it are the headers.
    Obviously we don't store any pixels, just the starting offset.
    This is quite different from the standard Image struct, since it is used for bufferized processing.
*/
pub struct BinaryImage {
    pub reader: BufReader<File>,
    pub magic_number: String,
    pub pixels_offset: usize,
    pub height: usize,
    pub width: usize,
    pub rgb_max_value: usize,
}

// Our implemented image transformation processes.
// These values must be used with process_and_output() second parameter.
pub enum ImageProcess {
    Invert,
    Greyscale,
}

impl BinaryImage {

    /*
        process_and_output(filename, process)

        Process (transform) image and output it to a file.

        The first parameter is the output's filename.
        The second parameter is one of ImageProcess enum's values, it tells wiich transformation that we want to apply.

        Return a Result with nothing if everything went smooth.

        WARNING: this is certainly not thread safe when being called from any context. but it's using threads, so that's cool.
    
    */
    pub fn process_and_output(&mut self, filename: &Path, process: ImageProcess) -> Result<(), Error> {
        
        /* 
            We were forced to use an unsafe bloc here because the main buffer was not effectively
            written over by our threads. No compile error, no runtime error, just nothing happening.
            We still need to investigate the issue cause it seems really likely that we missed something
            about memory and thread concurrency in Rust.

            Still working on it. 
        */
        unsafe {

            // Seek pixels section starting position.
            match self.reader.seek(SeekFrom::Start(self.pixels_offset as u64)) {
                Ok(file) => file,
                Err(e) => return Err(e),
            };

            // Create our output file.
            let out_file = match File::create(filename) {
                Ok(file) => file,
                Err(e) => return Err(e),
            };

            // Get a writer to the output file.
            let mut writer = BufWriter::new(out_file);

            /*  Write headers to the output file */
            
            // Magic number
            writer.write(self.magic_number.as_bytes()).unwrap();
            writer.write(b"\n").unwrap();

            // Width and height
            writer.write(self.width.to_string().as_bytes()).unwrap();
            writer.write(b" ").unwrap();
            writer.write(self.height.to_string().as_bytes()).unwrap();
            writer.write(b"\n").unwrap();

            // RGB max value.
            writer.write(self.rgb_max_value.to_string().as_bytes()).unwrap();
            writer.write(b"\n").unwrap();
            
            // get right function for processing.
            let func = match process {
                ImageProcess::Invert => invert_binary_pixel,
                ImageProcess::Greyscale => greyscale_binary_pixel,
            };

            // the pixels section size in bytes.
            let mut number_of_pixels_bytes = (self.width * self.height) * 3;

            /* 
                How many threads we should be able to launch for one iteration of the main loop.
                this is actually the number of logical processing units (threads) in our CPU and jere it is 
                retrieved at runtime.
            */
            let cores = num_cpus::get();

            /* Here comes the main reading -> spawning threads -> processing -> writing loop. */

            // read until there's no more pixel bytes.
            while number_of_pixels_bytes > 0 {

                /* 
                    our main buffer where we do the processing.
                    3 megabytes buffer seems to be a good starting point 
                */
                const PIXELS_BUFFER_BYTES_LENGTH: usize = 1024 * 1024 * 3;
                static mut BUFFER: [u8; PIXELS_BUFFER_BYTES_LENGTH] = [0; PIXELS_BUFFER_BYTES_LENGTH];

                // compute how many bytes we must read.
                // read size of our buffer if there's more or equal to our buffer size in file.
                let bytes_read = if number_of_pixels_bytes >= PIXELS_BUFFER_BYTES_LENGTH {
                    PIXELS_BUFFER_BYTES_LENGTH
                }
                // else, read what's left in file.
                else {
                    number_of_pixels_bytes
                };

                for i in 0..PIXELS_BUFFER_BYTES_LENGTH {
                    BUFFER[i] = 0;
                }

                // fill the buffer from our file
                match self.reader.read_exact(&mut BUFFER[..bytes_read]) {
                    Ok(read) => read,
                    Err(e) => {
                        return Err(e);
                    }
                };

                // How many pixels did we just read ?
                let pixels_read = bytes_read / 3;

                // We need to compute how many threads we must spawn
                let to_spawn = if pixels_read < cores {
                    pixels_read
                } else {
                    cores
                };

                // How many pixels each thread should work with ?
                // This will be used for buffer slicing inside the thread closure.
                let pixels_per_thread = pixels_read / to_spawn;

                // This vector will hold our thread handlers to be used after spawning with handler[i]join().
                let mut handles = vec![];

                // spawn our threads and push handlers to our vec.
                for i in 0..to_spawn {
                    
                    handles.push(std::thread::spawn(move || {
                        
                        // Get a buffer's slice for each thread.
                        let slice = &mut BUFFER
                            [(i * (pixels_per_thread * 3))..((i + 1) * (pixels_per_thread * 3))];

                        // For each 3 bytes in the slice, which is 1 pixel.
                        for y in 0..(slice.len() / 3) {

                            // Get a BinaryPixel struct from our position in buffer.
                            let mut pixel = BinaryPixel {
                                r: slice[y * 3] as u32,
                                g: slice[(y * 3) + 1] as u32,
                                b: slice[(y * 3) + 2] as u32,
                            };

                            // Transform pixel.
                            func(&mut pixel);

                            // Overwrite pixels values in our buffer with new transformed values.
                            slice[y * 3] = pixel.r as u8;
                            slice[(y * 3) + 1] = pixel.g as u8;
                            slice[(y * 3) + 2] = pixel.b as u8;
                        }
                    }));
                }

                // Wait for threads to finish before next iteration.
                for handle in handles {
                    handle.join().unwrap();
                }

                // Write the transformed buffer to the output file
                match writer.write_all(&mut BUFFER[..bytes_read]) {
                    Ok(write) => write,
                    Err(e) => { return Err(e); }
                }

                // Substract all bytes that were processed.
                number_of_pixels_bytes = number_of_pixels_bytes - bytes_read;
            }

            Ok(())
        }
    }
}

/*

    This function is used by new_with_file_bin()

    It provides a way to get the next header string when parsing a .ppm file.
    It takes the reader the as input and will be seeking until the next section (header or pixels beginning offset)

    Returns the header that was found, or will error out.
    
    WARNING: Using it after all headers were read will result in undefined behavior !

*/

fn get_next_header(reader: &mut BufReader<File>) -> Result<String, Error> {
    
    // we should not need more than this static size buffer to parse a single header value.
    let mut buffer: [u8; 8192] = [0; 8192];
    let mut index: usize = 0;
    let mut end_of_header: usize = 0;

    let delimiters = vec!['\n', '\r', ' ', '\t'];
    let comment_delimiters = vec!['\n', '\r'];

    let parse_error = |msg: &str| Error::new(ErrorKind::InvalidInput, msg);

    // Get current offset in reader
    let current = match reader.seek(SeekFrom::Current(0)) {
        Ok(position) => position,
        Err(e) => {
            let msg = format!("Couldn't get current position ({}).", e.to_string());
            return Err(parse_error(&msg));
        }
    };

    // Fill buffer from file.
    match reader.read(&mut buffer[..]) {
        Ok(read) => read,
        Err(e) => {
            let msg = format!("Couldn't read from file ({}).", e.to_string());
            return Err(parse_error(&msg));
        }
    };
    // Find next whitespace
    for byte in buffer.iter() {
        let c = *byte as char;
        if delimiters.contains(&c) {
            end_of_header = index;
            break;
        };

        index = index + 1;
    }

    // Parse header to string
    let header_buffer = &buffer[..end_of_header];
    let header_string = match String::from_utf8(header_buffer.to_vec()) {
        Ok(s) => s.to_string(),
        Err(_e) => return Err(parse_error("Couldn't convert header to string.")),
    };

    // Get a buffer's slice after the header (starting at the fist whitespace character)
    let after_header = &buffer[end_of_header..];
    index = end_of_header;

    // Ignore comments, get to the next line.
    if header_string.starts_with('#') {

        // iterate until we find a newline (CR/LF) character.
        for byte in after_header.iter() {
            let c = *byte as char;

            if comment_delimiters.contains(&c) {
                match reader.seek(SeekFrom::Start(current + 1 + (index as u64))) {
                    Ok(position) => position,
                    Err(_e) => return Err(parse_error("Couldn't seek to new position.")),
                };

                // recurse function cause we just read a comment.
                return get_next_header(reader);
            }

            index = index + 1;
        }

        return Err(parse_error("Couldn't find end of comment."));
    }

    // Get to the next header offset in reader.
    for byte in after_header.iter() {
        let c = *byte as char;

        // Stop if we just met a non-whitespace character.
        if !delimiters.contains(&c) {
            match reader.seek(SeekFrom::Start(current + (index as u64))) {
                Ok(_position) => { break; }
                Err(_e) => return Err(parse_error("Couldn't seek to new position.")),
            }
        };

        index = index + 1;
    }

    Ok(header_string)
}


/*
    new_with_file_bin(filename)

    Create a new BinaryImage structure for an input file, this struct could then be used with process_and_output(out_filename, process).

    The only parameter is the path to the input file.

    Will return Result with Err if an error occurs.
*/

pub fn new_with_file_bin(filename: &Path) -> Result<BinaryImage, Error> {

    let file_error = |msg: &str| Error::new(ErrorKind::InvalidInput, msg);

    let file = match File::open(filename) {
        Ok(file) => file,
        Err(e) => {
            let msg = format!(
                "Could not read input .ppm file ({}).",
                e.to_string()
            );
            return Err(file_error(&msg));
        },
    };

    let mut reader = BufReader::new(file);

    // Create our struct by parsing headers in the right order.
    let image = BinaryImage {

        // Get magic number
        magic_number: match get_next_header(&mut reader) {
            Ok(magic_number) => {

                // Check if value is indeed P6.
                if magic_number != "P6" {
                    return Err(file_error("Binary PPM must have P6 as a magic number."));
                }
                magic_number
            }

            Err(e) => return Err(e),
        },

        // Get width header
        width: match get_next_header(&mut reader) {
            Ok(width_string) => match width_string.parse::<usize>() {
                Ok(parsed) => parsed,
                Err(e) => {
                    let msg = format!(
                        "Could not parse width header into a number ({}).",
                        e.to_string()
                    );
                    return Err(file_error(&msg));
                }
            },

            Err(e) => return Err(e),
        },

        // Get height header
        height: match get_next_header(&mut reader) {
            Ok(height_string) => match height_string.parse::<usize>() {
                Ok(parsed) => parsed,
                Err(e) => {
                    let msg = format!(
                        "Could not parse height header into a number ({}).",
                        e.to_string()
                    );
                    return Err(file_error(&msg));
                }
            },

            Err(e) => return Err(e),
        },

        // Get RGB max value
        rgb_max_value: match get_next_header(&mut reader) {
            Ok(rgb_max_value_string) => {
                let parsed = match rgb_max_value_string.parse::<usize>() {
                    Ok(parsed) => { 

                        // Check if value is 255.
                        if parsed != 255 {
                            let msg = format!(
                                "Only 24 bits pixels format is supported for now ({} as RGB_MAX_VALUE).",
                                parsed.to_string()
                            );
                            return Err(file_error(&msg));
                        }
                        parsed 
                    },
                    Err(e) => {

                        let msg = format!(
                            "Could not parse RGB max value header into a number ({}).",
                            e.to_string()
                        );
                        return Err(file_error(&msg));
                    }
                };

                if parsed != 255 {
                    return Err(file_error("Only 255 RGB max value is supported for now."));
                } else {
                    parsed
                }
            }

            Err(e) => return Err(e),
        },

        // Pixel offset should be right after the last header. 
        // Whitespace is supposed to be consumed as well so we can just seek from current position to get the pixels section offset.
        pixels_offset: match reader.seek(SeekFrom::Current(0)) {
            Ok(offset) => offset as usize,
            Err(e) => {
                let msg = format!("Couldn't get pixels offset ({}).", e.to_string());
                return Err(file_error(&msg));
            }
        },

        reader,
    };

    Ok(image)
}

// Module for testing and benchmarking
#[cfg(test)]
mod bench {

    use super::*;
    use test::Bencher;

    fn get_test_file_path() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/p6/test/alaska.ppm"))
    }

    #[test]
    fn test_invert() {

        const BUFFER_SIZE: usize = 8192 * 3;
        
        let in_file_path = get_test_file_path();
        let out_file_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/p6/test/invert.test.ppm"));

        let mut img = new_with_file_bin(&in_file_path).unwrap();
        img.process_and_output(&out_file_path, ImageProcess::Invert).unwrap();

        let in_file = File::open(&in_file_path).unwrap();
        let out_file = File::open(&out_file_path).unwrap();

        let mut in_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut out_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let mut in_file_reader = BufReader::new(in_file);
        let mut out_file_reader = BufReader::new(out_file); 

        let mut bytes_count: usize = (img.height * img.width) * 3;
        
        in_file_reader.seek(SeekFrom::Current(img.pixels_offset as i64)).unwrap();
        out_file_reader.seek(SeekFrom::Current(img.pixels_offset as i64)).unwrap();

        while bytes_count > 0 {

            let to_read = if bytes_count >= BUFFER_SIZE {
                BUFFER_SIZE
            } else {
                bytes_count
            };
            
            in_file_reader.read_exact(&mut in_buffer[..to_read]).unwrap(); 
            out_file_reader.read_exact(&mut out_buffer[..to_read]).unwrap();

            for i in 0..to_read {
                assert_eq!(255 - in_buffer[i], out_buffer[i]);
            }

            bytes_count = bytes_count - to_read;
        }
    }

    #[test]
    fn test_greyscale() {
        
        const BUFFER_SIZE: usize = 8192 * 3;
        
        let in_file_path = get_test_file_path();
        let out_file_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/p6/test/greyscale.test.ppm"));

        let mut img = new_with_file_bin(&in_file_path).unwrap();
        img.process_and_output(&out_file_path, ImageProcess::Greyscale).unwrap();

        let in_file = File::open(&in_file_path).unwrap();
        let out_file = File::open(&out_file_path).unwrap();

        let mut in_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let mut out_buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let mut in_file_reader = BufReader::new(in_file);
        let mut out_file_reader = BufReader::new(out_file); 

        let mut bytes_count: usize = (img.height * img.width) * 3;
        
        in_file_reader.seek(SeekFrom::Current(img.pixels_offset as i64)).unwrap();
        out_file_reader.seek(SeekFrom::Current(img.pixels_offset as i64)).unwrap();

        while bytes_count > 0 {

            let to_read = if bytes_count >= BUFFER_SIZE {
                BUFFER_SIZE
            } else {
                bytes_count
            };
            
            in_file_reader.read_exact(&mut in_buffer[..to_read]).unwrap(); 
            out_file_reader.read_exact(&mut out_buffer[..to_read]).unwrap();

            for i in 0..(to_read / 3) {

                let in_pixel = BinaryPixel {
                    r: in_buffer[i * 3] as u32,
                    g: in_buffer[(i * 3) + 1] as u32,
                    b: in_buffer[(i * 3) + 2] as u32,
                };

                let grey: u32 = (in_pixel.r + in_pixel.b + in_pixel.g) / 3;

                assert_eq!(grey, out_buffer[(i * 3)] as u32);
                assert_eq!(grey, out_buffer[(i * 3) + 1] as u32);
                assert_eq!(grey, out_buffer[(i * 3) + 2] as u32);
            }

            bytes_count = bytes_count - to_read;
        }
    }

   

    #[bench]
    fn bench_greyscale_image(b: &mut Bencher) {
        let in_file_path = get_test_file_path();
        let out_file_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/p6/test/greyscale.bench.ppm"));

        let mut img = new_with_file_bin(&in_file_path).unwrap();
        
        b.iter(|| img.process_and_output(&out_file_path, ImageProcess::Greyscale).unwrap());
    }

    #[bench]
    fn bench_invert_image(b: &mut Bencher) {
        let in_file_path = get_test_file_path();
        let out_file_path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/p6/test/invert.bench.ppm"));

        let mut img = new_with_file_bin(&in_file_path).unwrap();

        b.iter(|| img.process_and_output(&out_file_path, ImageProcess::Invert).unwrap());
    }
}
