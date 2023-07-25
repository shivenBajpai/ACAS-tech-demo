#![allow(dead_code)]
#![allow(unused)]
use std::f64::consts;
use rotsprite;
use image::{io::Reader, ImageResult, RgbaImage};
use acas::stitch;

fn main() {

    let input_image = Reader::open("examples/testing/spear.png").unwrap().decode().unwrap().into_rgba8();
    let (width,height) = input_image.dimensions();

    let input_buffer = input_image.into_vec();

    let (width,height,output_buffer) = stitch::rotate(input_buffer.as_slice(), &[0,0,0,0],  4, width as u64, height as u64, consts::PI/4.0).unwrap();

    let output_image = RgbaImage::from_vec(width,height,output_buffer).unwrap();

    output_image.save_with_format("examples/testing/output.png", image::ImageFormat::Png).expect("Failed to save image")
}