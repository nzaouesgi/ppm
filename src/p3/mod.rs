extern crate test;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::Path;

// use libc::double;
#[derive(Copy, Clone)]
pub struct Pixel {
    red: u8,
    blue: u8,
    green: u8,
}

pub struct Image {
    pixels: Vec<Pixel>,
    heigth: usize,
    width: usize,
    rgb_type: String,
    max_val: u8,
}

impl Pixel {

    pub fn invert(&mut self) {
        self.red = 255 - self.red;
        self.blue = 255 - self.blue;
        self.green = 255 - self.green;
    }

    pub fn eq(self, other: Pixel) -> bool {
        return self.red == other.red && self.blue == other.blue && self.green == other.green;
    }

    pub fn greyscale(&mut self) {
        let gris: u32 = ((self.red as u32 + self.blue as u32 + self.green as u32) / 3) as u32;
        self.red = (gris) as u8;
        self.blue = (gris) as u8;
        self.green = (gris) as u8;
    }
}

impl PartialEq for Pixel {
    fn eq(&self, other: &Self) -> bool {
        self.red == other.red && self.blue == other.blue && self.green == other.green
    }
}

//  Function that read in text mode a ppm image
pub fn new_with_file(filename: &Path) -> Image {
    
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let mut final_pixels: Vec<Pixel> = vec![];
    let mut final_height: usize = 0;
    let mut final_width: usize = 0;
    let mut final_max_value: u8 = 0;
    let mut type_rgb = String::from("");
    
    for (_index, line) in reader.lines().enumerate() {
        
        let line = line.unwrap();

        let parsed_line: Vec<&str> = line.trim().split(" ").collect();
        if parsed_line[0].chars().next().unwrap() != '#' {
            if parsed_line.len() == 2 {
                final_height = parsed_line[0].parse::<usize>().unwrap();
                final_width = parsed_line[1].parse::<usize>().unwrap();
            } else if parsed_line.len() >= 3 {
                
                let mut index = 0;
                let mut redf: u8 = 0;
                let mut bluef: u8 = 0;
                let mut greenf: u8 = 0;

                for i in 0..parsed_line.len() {
                    if parsed_line[i].is_empty() == false
                        && parsed_line[i].find(|c: char| (c < '0') || (c > '9')) == None
                    {
                        //println!("{} {}\n", parsedLine[i], i);
                        match index {
                            // Match a single value
                            0 => redf = parsed_line[i].parse::<u8>().unwrap(),
                            1 => bluef = parsed_line[i].parse::<u8>().unwrap(),
                            2 => greenf = parsed_line[i].parse::<u8>().unwrap(),
                            _ => redf = redf + 0,
                        }

                        index = index + 1;

                        if index == 3 {
                            let final_pixel = Pixel {
                                red: redf,
                                blue: bluef,
                                green: greenf,
                            };

                            final_pixels.push(final_pixel);
                            index = 0;
                        }
                    }
                }
            } else if parsed_line.len() == 1 {
                if parsed_line[0].find(|c: char| (c < '0') || (c > '9')) == None {
                    final_max_value = parsed_line[0].parse::<u8>().unwrap();
                } else {
                    type_rgb = parsed_line[0].to_string();
                }
            }
        }
        // still lines wich contains # or the of RGB color
    }
    return Image {
        pixels: final_pixels,
        heigth: final_height,
        width: final_width,
        rgb_type: type_rgb,
        max_val: final_max_value,
    };
}

impl Image {
    //  saves Image into a file
    pub fn save(&self, filename: &Path) {
        let mut file = File::create(filename).unwrap();
        file.write_all(self.rgb_type.as_bytes()).unwrap();
        file.write_all(b"\n").unwrap();
        let second_line = String::from(self.heigth.to_string() + " " + &self.width.to_string());
        file.write_all(second_line.as_bytes()).unwrap();
        file.write_all(b"\n").unwrap();
        file.write_all(self.max_val.to_string().as_bytes()).unwrap();
        file.write_all(b"\n").unwrap();
        for i in 0..self.pixels.len() {
            let pixel_line = String::from(
                self.pixels[i].red.to_string()
                    + " "
                    + &self.pixels[i].blue.to_string()
                    + " "
                    + &self.pixels[i].green.to_string(),
            );
            file.write_all(pixel_line.as_bytes()).unwrap();
            file.write_all(b"\n").unwrap();
        }
        //close the file
        drop(file);
    }

    //    function that inverts image colors
    pub fn invert(&mut self) {
        for i in 0..self.pixels.len() {
            self.pixels[i].invert();
        }
    }
    //    function that makes image B&W based on a filter color
    pub fn greyscale(&mut self) {
        for i in 0..self.pixels.len() {
            self.pixels[i].greyscale();
        }
    }
}

//Tests and Benchmark
#[cfg(test)]
mod bench {
    use super::*;
    use test::Bencher;

    fn get_test_file_path() -> &'static str {
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/p3/test/test.ppm")
    }

    fn get_test_output_file_path() -> &'static str {
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/p3/test/test.output.ppm")
    }

    #[test]
    fn test_invert() {
        let test_file = get_test_file_path();

        let mut image = new_with_file(Path::new(&test_file));
        let image_aux = new_with_file(Path::new(&test_file));

        image.invert();

        for i in 0..image.pixels.len() {
            // self.pixels[i].invert();
            assert_eq!(255 - image.pixels[i].red, image_aux.pixels[i].red);
            assert_eq!(255 - image.pixels[i].blue, image_aux.pixels[i].blue);
            assert_eq!(255 - image.pixels[i].green, image_aux.pixels[i].green);
        }
    }

    #[test]
    fn test_greyscale() {
        let test_file = Path::new(get_test_file_path());

        let mut image = new_with_file(&test_file);
        let image_aux = new_with_file(&test_file);

        image.greyscale();

        for i in 0..image.pixels.len() {
            // self.pixels[i].invert();
            assert_eq!(
                (image_aux.pixels[i].red as u32
                    + image_aux.pixels[i].blue as u32
                    + image_aux.pixels[i].green as u32)
                    / 3,
                image.pixels[i].red as u32
            );
            assert_eq!(
                (image_aux.pixels[i].red as u32
                    + image_aux.pixels[i].blue as u32
                    + image_aux.pixels[i].green as u32)
                    / 3,
                image.pixels[i].blue as u32
            );
            assert_eq!(
                (image_aux.pixels[i].red as u32
                    + image_aux.pixels[i].blue as u32
                    + image_aux.pixels[i].green as u32)
                    / 3,
                image.pixels[i].green as u32
            );
        }
    }

    #[bench]
    fn bench_create_file(b: &mut Bencher) {
        let test_file = Path::new(get_test_file_path());

        b.iter(|| new_with_file(&test_file));
    }

    #[bench]
    fn bench_output_file(b: &mut Bencher) {
        let test_file = Path::new(get_test_file_path());
        let test_file_output = Path::new(get_test_output_file_path());

        let image = new_with_file(&test_file);
        b.iter(|| image.save(&test_file_output));
    }

    #[bench]
    fn bench_greyscale_image(b: &mut Bencher) {
        let test_file = Path::new(get_test_file_path());
        let test_file_output = Path::new(get_test_output_file_path());

        let mut image = new_with_file(&test_file);
        image.greyscale();
        b.iter(|| image.save(&test_file_output));
    }

    #[bench]
    fn bench_invert_image(b: &mut Bencher) {
        let test_file = Path::new(get_test_file_path());
        let test_file_output = Path::new(get_test_output_file_path());

        let mut image = new_with_file(&test_file);
        image.invert();
        b.iter(|| image.save(&test_file_output));
    }
}
