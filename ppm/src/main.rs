extern crate libc;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::LineWriter;
use std::io::{BufRead, BufReader};
use std::path::Path;
// use libc::double;
#[derive(Copy, Clone)]
struct Pixel {
    rouge :u8,
    bleu: u8,
    vert: u8
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
            rouge: red,
            vert: green,
            bleu: blue
        };
    }

    fn red(&self) -> u8 {
        return self.rouge;
    }

    fn bleu(&self) -> u8 {
        return self.bleu;
    }

    fn vert(&self) -> u8 {
        return self.vert;
    }

    fn display(&self) {
        println!("Rouge = {}\nBleu = {}\nVert = {}", self.rouge, self.bleu, self.vert);
    }

    fn invert(&mut self) {
        self.rouge = 255 - self.rouge;
        self.bleu = 255 - self.bleu;
        self.vert = 255 - self.vert;
    }

    fn eq(self, other: Pixel) -> bool {
        return self.rouge == other.rouge && self.bleu == other.bleu && self.vert == other.vert;
    }

    fn grayscale(&mut self) ->Pixel {
        let gris = (self.rouge + self.bleu + self.vert) / 3;
        return Pixel {
            rouge: gris,
            bleu: gris,
            vert: gris
        }
    }
}


impl PartialEq for Pixel {
    fn eq(&self, other: &Self) -> bool {
        self.rouge == other.rouge && self.bleu == other.bleu && self.vert == other.vert
    }
}


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
                    let mut rougef : u8 = 0;
                    let mut bleuf : u8 = 0;
                    let mut vertf : u8 = 0;
                    for i in 0..parsedLine.len() {
                        if parsedLine[i].is_empty() == false && parsedLine[i].find(|c: char| (c < '0') || (c > '9')) == None {
                            //println!("{} {}\n", parsedLine[i], i);
                            match index {
                                // Match a single value
                                0 => rougef = parsedLine[i].parse::<u8>().unwrap(),
                                1 => bleuf = parsedLine[i].parse::<u8>().unwrap(),
                                2 => vertf = parsedLine[i].parse::<u8>().unwrap(),
                                _ => rougef = rougef + 0
                            }
                            index = index + 1;
                            if index == 3 {
                                let finalPixel = Pixel {
                                    rouge : rougef,
                                    bleu:  bleuf,
                                    vert: vertf
                                };
                    // println!("Rouge = {}\nBleu = {}\nVert = {}", finalPixel.rouge, finalPixel.bleu, finalPixel.vert);
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
        let pixelLine = String::from(self.pixels[i].rouge.to_string()+" "+&self.pixels[i].bleu.to_string()+" "+&self.pixels[i].vert.to_string());
        file.write_all(pixelLine.as_bytes()).unwrap();
        file.write_all(b"\n");
      }
      //close the file
      drop(file);
  }
}







fn main() {
    let x = unsafe { sin(10) };
    let image = new_with_file(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/test.txt"));
    let output = Image { heigth: image.heigth, width: image.width, pixels: image.pixels, rgbType: image.rgbType, maxVal: image.maxVal};
    output.save(Path::new("/home/moxa/Bureau/ESGI/Programmation système et réseau/ppm/ppm/src/res.txt"));
    println!("double de 100 {}", x);
}
