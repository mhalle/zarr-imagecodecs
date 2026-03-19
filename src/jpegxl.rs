use std::io::Cursor;

pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let image = jxl_oxide::JxlImage::builder()
        .read(Cursor::new(data))?;

    let render = image.render_frame(0)?;
    let fb = render.image_all_channels();

    // Convert f32 framebuffer to u8
    let pixels: Vec<u8> = fb
        .buf()
        .iter()
        .map(|&v| (v.clamp(0.0, 1.0) * 255.0) as u8)
        .collect();

    Ok(pixels)
}
