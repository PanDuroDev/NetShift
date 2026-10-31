use std::{error::Error, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=icons/icon.png");
    println!("cargo:rerun-if-changed=icons/icon.ico");
    ensure_default_icons().expect("failed to generate the NetShift prototype icons");
    tauri_build::build();
}

fn ensure_default_icons() -> Result<(), Box<dyn Error>> {
    let icon_directory = Path::new("icons");
    fs::create_dir_all(icon_directory)?;

    let png_path = icon_directory.join("icon.png");
    let ico_path = icon_directory.join("icon.ico");

    if !png_path.exists() {
        let pixels = render_icon_pixels(256);
        let file = fs::File::create(&png_path)?;
        let mut encoder = png::Encoder::new(file, 256, 256);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&pixels)?;
        drop(writer);
    }

    if !ico_path.exists() {
        let mut icon_directory = ico::IconDir::new(ico::ResourceType::Icon);

        for size in [16_u32, 24, 32, 48, 64, 256] {
            let pixels = render_icon_pixels(size);
            let image = ico::IconImage::from_rgba_data(size, size, pixels);
            icon_directory.add_entry(ico::IconDirEntry::encode(&image)?);
        }

        let file = fs::File::create(&ico_path)?;
        icon_directory.write(file)?;
    }

    Ok(())
}

fn render_icon_pixels(size: u32) -> Vec<u8> {
    let mut pixels = vec![0_u8; (size * size * 4) as usize];
    let scale = size as f32 / 64.0;

    for y in 0..size {
        for x in 0..size {
            let index = ((y * size + x) * 4) as usize;
            let design_x = (x as f32 + 0.5) / scale;
            let design_y = (y as f32 + 0.5) / scale;
            let inside = rounded_square(design_x, design_y, 64.0, 13.0);
            let is_border = inside && !rounded_square(design_x, design_y, 64.0, 12.0);

            let color = if !inside {
                [0, 0, 0, 0]
            } else if is_border {
                [51, 101, 110, 255]
            } else if on_segment(design_x, design_y, 19.0, 32.0, 44.0, 18.0, 1.8)
                || on_segment(design_x, design_y, 19.0, 32.0, 44.0, 46.0, 1.8)
            {
                [84, 203, 216, 255]
            } else if in_circle(design_x, design_y, 19.0, 32.0, 5.2)
                || in_circle(design_x, design_y, 44.0, 18.0, 5.2)
                || in_circle(design_x, design_y, 44.0, 46.0, 5.2)
            {
                [141, 229, 238, 255]
            } else {
                [16, 21, 24, 255]
            };

            pixels[index..index + 4].copy_from_slice(&color);
        }
    }

    pixels
}

fn rounded_square(x: f32, y: f32, size: f32, radius: f32) -> bool {
    let nearest_x = x.clamp(radius, size - radius - 1.0);
    let nearest_y = y.clamp(radius, size - radius - 1.0);
    let dx = x - nearest_x;
    let dy = y - nearest_y;

    dx * dx + dy * dy <= radius * radius
}

fn in_circle(x: f32, y: f32, center_x: f32, center_y: f32, radius: f32) -> bool {
    let dx = x - center_x;
    let dy = y - center_y;
    dx * dx + dy * dy <= radius * radius
}

fn on_segment(
    x: f32,
    y: f32,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    half_width: f32,
) -> bool {
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let length_squared = dx * dx + dy * dy;
    let position = (((x - start_x) * dx + (y - start_y) * dy) / length_squared)
        .clamp(0.0, 1.0);
    let nearest_x = start_x + position * dx;
    let nearest_y = start_y + position * dy;
    let offset_x = x - nearest_x;
    let offset_y = y - nearest_y;

    offset_x * offset_x + offset_y * offset_y <= half_width * half_width
}
