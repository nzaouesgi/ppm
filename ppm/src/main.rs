#![feature(test)]
extern crate test;
extern crate libc;
use std::fs::{File};
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::Path;
// use libc::double;
#[derive(Copy, Clone)]
struct Pixel {
    red :u8,
    blue: u8,
    green: u8
}

struct Image {
    pixels: Vec<Pixel>,
    heigth: usize,
    width: usize,
    rgbType: String,
    maxVal: u8
}

#[link(name = "c")]
extern {
    fn sin(x: u32) -> u32;
}
//impl Pixel: Copy + Clone{
impl Pixel {
    fn new(red: u8, green: u8, blue: u8) -> Pixel{
        return Pixel {
            red: red,
            green: green,
            blue: blue
        };
    }

    fn red(&self) -> u8 {
        return self.red;
    }

    fn blue(&self) -> u8 {
        return self.blue;
    }

    fn green(&self) -> u8 {
        return self.green;
    }

    fn display(&self) {
        println!("red = {}\nblue = {}\ngreen = {}", self.red, self.blue, self.green);
    }

    fn invert(&mut self) {
        self.red = 255 - self.red;
        self.blue = 255 - self.blue;
        self.green = 255 - self.green;
    }

    fn eq(self, other: Pixel) -> bool {
        return self.red == other.red && self.blue == other.blue && self.green == other.green;
    }

    fn greyScale(&mut self) {
        let mut gris : u32 = ((self.red as u32 + self.blue as u32 + self.green as u32) / 3) as u32;
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
fn new_with_file(filename: &Path) -> Image {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let mut finalPixels : Vec<Pixel>  = vec![];
    let mut finalHeight : usize = 0;
    let mut finalWidth : usize = 0;
    let mut finalmaxVal : u8 = 0;
    let mut typeRgb = String::from("");
    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap();
            let parsedLine:Vec<&str> = line.trim().split(" ").collect();
            if parsedLine[0].chars().next().unwrap() != '#' {
                if parsedLine.len() == 2 {
                    finalHeight = parsedLine[0].parse::<usize>().unwrap();
                    finalWidth =  parsedLine[1].parse::<usize>().unwrap();
                } else if parsedLine.len() >= 3 {
                    // println!("{} {}  {}\n", parsedLine[0], parsedLine[1], parsedLine[2]);
                    let mut index = 0;
                    let mut redf : u8 = 0;
                    let mut bluef : u8 = 0;
                    let mut greenf : u8 = 0;
                    for i in 0..parsedLine.len() {
                        if parsedLine[i].is_empty() == false && parsedLine[i].find(|c: char| (c < '0') || (c > '9')) == None {
                            //println!("{} {}\n", parsedLine[i], i);
                            match index {
                                // Match a single value
                                0 => redf = parsedLine[i].parse::<u8>().unwrap(),
                                1 => bluef = parsedLine[i].parse::<u8>().unwrap(),
                                2 => greenf = parsedLine[i].parse::<u8>().unwrap(),
                                _ => redf = redf + 0
                            }
                            index = index + 1;
                            if index == 3 {
                                let finalPixel = Pixel {
                                    red : redf,
                                    blue:  bluef,
                                    green: greenf
                                };
                                finalPixels.push(finalPixel);
                                index = 0;
                            }
                        }
                    }
                } else if parsedLine.len() == 1 {
                    if parsedLine[0].find(|c: char| (c < '0') || (c > '9')) == None {
                        finalmaxVal = parsedLine[0].parse::<u8>().unwrap();
                    }
                    else {
                        typeRgb = parsedLine[0].to_string();
                    }
                }
            }
            // still lines wich contains # or the of RGB color
        }
    return Image {
        pixels : finalPixels,
        heigth: finalHeight,
        width: finalWidth,
        rgbType: typeRgb,
        maxVal: finalmaxVal
    }
}


impl Image {
  //  saves Image into a file
  fn save(&self, filename: &Path) {
      let mut file = File::create(filename).unwrap();
      file.write_all(self.rgbType.as_bytes()).unwrap();
      file.write_all(b"\n");
      let secondLine = String::from(self.heigth.to_string()+" "+&self.width.to_string());
      file.write_all(secondLine.as_bytes()).unwrap();
      file.write_all(b"\n");
      file.write_all(self.maxVal.to_string().as_bytes()).unwrap();
      file.write_all(b"\n");
      for i in 0..self.pixels.len() {
        let pixelLine = String::from(self.pixels[i].red.to_string()+" "+&self.pixels[i].blue.to_string()+" "+&self.pixels[i].green.to_string());
        file.write_all(pixelLine.as_bytes()).unwrap();
        file.write_all(b"\n");
      }
      //close the file
      drop(file);
  }

  //    function that inverts image colors
  fn invertColors(&mut self) {
    for i in 0..self.pixels.len() {
        self.pixels[i].invert();
      }
  }
  //    function that makes image B&W based on a filter color
  fn greyScale(&mut self) {
    for i in 0..self.pixels.len() {
        self.pixels[i].greyScale();
      }
  }
}


pub fn add_two(a: i32) -> i32 {
    a + 2
}


//Tests and Benchmark
#[cfg(test)]
mod bench {
    use super::*;
    use test::Bencher;

    #[test]
    fn invertColors() {
        let mut image = new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/test.txt"));
        let imageAux = new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/test.txt"));

        image.invertColors();

        for i in 0..image.pixels.len() {
            // self.pixels[i].invert();
            assert_eq!(255 - image.pixels[i].red, imageAux.pixels[i].red);
            assert_eq!(255 - image.pixels[i].blue, imageAux.pixels[i].blue);
            assert_eq!(255 - image.pixels[i].green, imageAux.pixels[i].green);
          }
    }

    #[test]
    fn greyScale() {
        let mut image = new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/test.txt"));
        let imageAux = new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/test.txt"));

        image.greyScale();

        for i in 0..image.pixels.len() {
            // self.pixels[i].invert();
            assert_eq!((imageAux.pixels[i].red as u32 + imageAux.pixels[i].blue as u32 + imageAux.pixels[i].green as u32) / 3, image.pixels[i].red as u32);
            assert_eq!((imageAux.pixels[i].red as u32 + imageAux.pixels[i].blue as u32 + imageAux.pixels[i].green as u32) / 3, image.pixels[i].blue as u32);
            assert_eq!((imageAux.pixels[i].red as u32 + imageAux.pixels[i].blue as u32 + imageAux.pixels[i].green as u32) / 3, image.pixels[i].green as u32);
          }
    }


    #[bench]
    fn bench_createImage_from_file(b: &mut Bencher) {
        //  let image = new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/test.txt"));
        b.iter(|| new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/test.txt")));
    }

    #[bench]
    fn transformingImage_into_file(b: &mut Bencher) {
        let image = new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/test.txt"));
        //  let output = Image { heigth: image.heigth, width: image.width, pixels: image.pixels, rgbType: image.rgbType, maxVal: image.maxVal};
        b.iter(|| image.save(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/res.txt")));
    }


    #[bench]
    fn greyScaleOfAnImage(b: &mut Bencher) {
        let mut image = new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/test.txt"));
        image.greyScale();
        b.iter(|| image.save(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/res.txt")));
    }

    #[bench]
    fn invertColorsOfAnImage(b: &mut Bencher) {
        let mut image = new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/test.txt"));
        image.invertColors();
        b.iter(|| image.save(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/res.txt")));
    }
}




fn main() {
    let mut image = new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/lol.ppm"));
    image.invertColors();
    image.save(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/res.ppm"));
}
