use std::{fs, path::Path};

use clap::Parser;
use image::{GenericImageView, ImageReader, Pixel, Rgb, RgbImage, imageops::FilterType};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    path: Vec<String>,
}

fn main() {
    let cli = Cli::parse();
    for p in cli.path {
        let p = Path::new(&p);
        if p.is_dir() {
            continue;
        }
        let o_dir = p.parent().unwrap().join("out");
        let o = o_dir.join(p.file_name().unwrap());
        if let Result::Ok(false) = fs::exists(&o_dir) {
            fs::create_dir(&o_dir).unwrap();
        }
        process(p, &o);
    }
}

fn process(i: &Path, o: &Path) {
    let img = ImageReader::open(i).unwrap().decode().unwrap();
    let background = img
        .resize_to_fill(1080, 1080, FilterType::CatmullRom)
        .blur(40.0);
    let foreground = img.resize(1080, 1080, FilterType::CatmullRom);

    let (wb, hb) = background.dimensions();
    let (wf, hf) = foreground.dimensions();

    let (fore_x1, fore_y1, fore_x2, fore_y2) = if wf > hf {
        (0, hb / 2 - hf / 2, 1080, hb / 2 + hf / 2)
    } else {
        (wb / 2 - wf / 2, 0, wb / 2 + wf / 2, 1080)
    };

    let mut result = RgbImage::new(1080, 1080);

    for (x, y, pixel) in result.enumerate_pixels_mut() {
        if fore_x1 <= x && x < fore_x2 && fore_y1 <= y && y < fore_y2 {
            // if x - fore_x1 >= wf || y - fore_y1 >= hf {
            //     continue;
            // }
            *pixel = foreground.get_pixel(x - fore_x1, y - fore_y1).to_rgb();
        } else {
            let data = background.get_pixel(x, y);
            *pixel = alpha_blending(data.to_rgb(), Rgb([255, 255, 255]), 0.3);
        }
    }

    result.save(o).unwrap();
}

fn alpha_blending(fore: Rgb<u8>, back: Rgb<u8>, alpha: f64) -> Rgb<u8> {
    fn alpha_blending_single(fore: u8, back: u8, alpha: f64) -> u8 {
        (fore as f64 * alpha + back as f64 * (1. - alpha)) as u8
    }

    Rgb([
        alpha_blending_single(fore[0], back[0], alpha),
        alpha_blending_single(fore[1], back[1], alpha),
        alpha_blending_single(fore[2], back[2], alpha),
    ])
}
