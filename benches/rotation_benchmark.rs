use criterion::{criterion_group, criterion_main, Criterion};
use rotsprite::rotate::rotate;
use acas::stitch;
use image::{io::Reader, RgbaImage,Rgba};
use std::f64::consts;

fn test_native<P> (input_buffer: &[P], empty: &[P], width: u32, height: u32) where P: Clone {
    let _ = stitch::rotate(input_buffer, empty, 4, width, height, consts::PI/4.0).unwrap();
}

fn test_rotsprite<P> (input_buffer: &[P], empty: &P, width: usize, height: usize) where P: Clone + Eq {
    let _ = rotate(input_buffer, empty, width, height, 45.0,1);
}

fn benchmark_native(c: &mut Criterion) {
    let input_image = Reader::open("benches/input.png").unwrap().decode().unwrap().into_rgba8();
    let (width,height) = input_image.dimensions();
    let input_buffer = input_image.into_vec();
    let input_slice = input_buffer.as_slice();

    c.bench_function("Acas Rotate spear", |b| b.iter(|| test_native(input_slice,&[0,0,0,0],width,height)));
}

fn benchmark_foreign(c: &mut Criterion) {
    let img = image::open("benches/input.png").unwrap();
    let width = img.width() as usize;
    let height = img.height() as usize;
    let image: &RgbaImage = img
        .as_rgba8()
        .expect("Could not convert image to RGBA8 array");

    let pixels: Vec<Rgba<u8>> = image.pixels().copied().collect();
    let unfound_color = Rgba([0, 0, 0, 0]);

    c.bench_function("Rotsprite Rotate spear", |b| b.iter(|| test_rotsprite(&pixels,&unfound_color,width,height)));
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = benchmark_native, benchmark_foreign
}
criterion_main!(benches);