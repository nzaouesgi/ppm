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
    rgbType: String
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
    let mut typeRgb = String::from("");
    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap(); 
            let parsedLine:Vec<&str>= line.split(" ").collect();
            if parsedLine.len() == 2 {
                finalHeight = parsedLine[0].parse::<usize>().unwrap();
                finalWidth =  parsedLine[1].parse::<usize>().unwrap();
            } else if parsedLine.len() == 3 {
                let linePixel:Vec<&str>= line.split(" ").collect();
      
                let finalPixel = Pixel {
                    rouge : linePixel[0].parse::<u8>().unwrap(),
                    bleu:  linePixel[1].parse::<u8>().unwrap(),
                    vert: linePixel[2].parse::<u8>().unwrap()
                };
                finalPixels.push(finalPixel);
            } else if parsedLine.len() == 1 {
                typeRgb = parsedLine[0].to_string();
            }
            // still lines wich contains # or the of RGB color
        }
    return Image {
        pixels : finalPixels,
        heigth: finalHeight,
        width: finalWidth,
        rgbType: typeRgb
    }
}


impl Image {
  fn save(&self, filename: &Path) {
      let mut file = File::create(filename).unwrap();
      file.write_all(self.rgbType.as_bytes()).unwrap();
      let secondLine = String::from(self.heigth.to_string()+" "+&self.width.to_string());
      file.write_all(secondLine.as_bytes()).unwrap();
      
      for i in 0..self.pixels.len() {
        let pixelLine = String::from(self.pixels[i].rouge.to_string()+" "+&self.pixels[i].bleu.to_string()+" "+&self.pixels[i].vert.to_string());
        file.write_all(pixelLine.as_bytes()).unwrap();
      }
      //close the file
      drop(file);
  }
}

fn main() {
    let x = unsafe { sin(10) };
    println!("double de 100 {}", x);
}
