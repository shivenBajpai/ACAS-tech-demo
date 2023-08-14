use std::f64::consts;
use acas::stitch;
use image::{io::Reader, RgbaImage, Pixel, Rgba, ImageResult};

// This Data will be automatically read from files in a later version
const SWORD_ANGLE: f64 = consts::PI*-27.0/180.0;
const SWORD_ANCHOR: (usize, usize) = (24,12);
const FRAME_ANCHORS: [(usize, usize); 3] = [(1,25),(5,34),(24,30)];
const FRAME_ANGLES: [f64; 3] = [consts::PI*-27.0/180.0, consts::PI*13.0/180.0, consts::PI*-27.0/180.0];

fn main() {
    let images = load_images();
    let sword = load_sword();
    let mut output = vec![];

    for image in 0..3 {
        output.push(stitch::stitch(
            images[image].0.as_slice(), // base buffer (texture to be stitched on)
            sword.0.as_slice(), // appendage buffer (texture to be stitched)
            &[0,0,0,0], // pixel used for filling in gaps
            4, // channels per pixel
            images[image].1, // base image dimensions
            FRAME_ANCHORS[image], // position on base buffer to place anchor pixel
            FRAME_ANGLES[image], // desired angle of appendage
            sword.1, // appendage image dimensions
            SWORD_ANCHOR, // position of anchor pixel on appendage buffer
            SWORD_ANGLE, // angle of appendage in given buffer 
            stitch::StitchingOrder::AppendageOnTop,
            stitch::StitchingQuality::Fancy
        ).unwrap());

        save_image(image, output[image].0 as u32, output[image].1 as u32, output[image].2.clone()).unwrap();
    }

    output.iter().enumerate().for_each(
        |(i,x)| save_image(i, x.0 as u32, x.1 as u32, x.2.clone()).unwrap()
    );
}

fn load_images() -> [(Vec<u8>, (usize, usize)); 3] {
    let frame_1 = Reader::open("examples/stitching/assets/frame1.png").unwrap().decode().unwrap().into_rgba8();
    let width1 = frame_1.dimensions().0 as usize;
    let height1 = frame_1.dimensions().1 as usize;
    let buf1 = frame_1.into_vec();

    let frame_2 = Reader::open("examples/stitching/assets/frame2.png").unwrap().decode().unwrap().into_rgba8();
    let width2 = frame_2.dimensions().0 as usize;
    let height2 = frame_2.dimensions().1 as usize;
    let buf2 = frame_2.into_vec();

    let frame_3 = Reader::open("examples/stitching/assets/frame3.png").unwrap().decode().unwrap().into_rgba8();
    let width3 = frame_3.dimensions().0 as usize;
    let height3 = frame_3.dimensions().1 as usize;
    let buf3 = frame_3.into_vec();

    [
        (buf1,(width1,height1)),
        (buf2,(width2,height2)),
        (buf3,(width3,height3))
    ]

}

fn load_sword() -> (Vec<u8>, (usize, usize)) {
    let appendage_image = Reader::open("examples/stitching/assets/sword.png").unwrap().decode().unwrap().into_rgba8();
    let widtha = appendage_image.dimensions().0 as usize;
    let heighta = appendage_image.dimensions().1 as usize;
    let bufa = appendage_image.into_vec();

    (bufa,(widtha,heighta))
}

fn save_image(index: usize, width: u32, height: u32, buf: Vec<<Rgba<u8> as Pixel>::Subpixel>) -> ImageResult<()> {
    let output_image = RgbaImage::from_vec(width,height,buf).unwrap();
    let path = format!("examples/stitching/output{}.png",index);
    output_image.save_with_format(path, image::ImageFormat::Png)?;

    Ok(())
}