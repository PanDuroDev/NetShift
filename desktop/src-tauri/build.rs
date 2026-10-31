use std::{error::Error, fs, path::Path};

fn main() {
    ensure_default_icon().expect("failed to generate the NetShift prototype icon");
    tauri_build::build();
}

fn ensure_default_icon() -> Result<(), Box<dyn Error>> {
    let icon_directory = Path::new("icons");
    fs::create_dir_all(icon_directory)?;

    let icon_path = icon_directory.join("icon.png");
    if icon_path.exists() {
        return Ok(());
    }

    const SIZE: u32 = 64;
    let mut pixels = vec![0_u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let index = ((y * SIZE + x) * 4) as usize;
            let inside = rounded_square(x, y, SIZE, 13);
            let is_border = inside && !rounded_square(x, y, SIZE, 12);

            let color = if !inside {
                [0, 0, 0, 0]
            } else if is_border {
                [51, 101, 110, 255]
            } else if on_segment(x, y, 19.0, 32.0, 44.0, 18.0, 1.8)
                || on_segment(x, y, 19.0, 32.0, 44.0, 46.0, 1.8)
            {
                [84, 203, 216, 255]
            } else if in_circle(x, y, 19.0, 32.0, 5.2)
                || in_circle(x, y, 44.0, 18.0, 5.2)
                || in_circle(x, y, 44.0, 46.0, 5.2)
            {
                [141, 229, 238, 255]
            } else {
                [16, 21, 24, 255]
            };

            pixels[index..index + 4].copy_from_slice(&color);
        }
    }

    let file = fs::File::create(icon_path)?;
    let mut encoder = png::Encoder::new(file, SIZE, SIZE);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn rounded_square(x: u32, y: u32, size: u32, radius: u32) -> bool {
    let x = x as i32;
    let y = y as i32;
    let size = size as i32;
    let radius = radius as i32;
    let nearest_x = x.clamp(radius, size - radius - 1);
    let nearest_y = y.clamp(radius, size - radius - 1);
    let dx = x - nearest_x;
    let dy = y - nearest_y;

    dx * dx + dy * dy <= radius * radius
}

fn in_circle(x: u32, y: u32, center_x: f32, center_y: f32, radius: f32) -> bool {
    let dx = x as f32 - center_x;
    let dy = y as f32 - center_y;
    dx * dx + dy * dy <= radius * radius
}

fn on_segment(
    x: u32,
    y: u32,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    half_width: f32,
) -> bool {
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let length_squared = dx * dx + dy * dy;
    let position = (((x as f32 - start_x) * dx + (y as f32 - start_y) * dy)
        / length_squared)
        .clamp(0.0, 1.0);
    let nearest_x = start_x + position * dx;
    let nearest_y = start_y + position * dy;
    let offset_x = x as f32 - nearest_x;
    let offset_y = y as f32 - nearest_y;

    offset_x * offset_x + offset_y * offset_y <= half_width * half_width
}
