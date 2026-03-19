use jpeg_encoder::{ColorType as EncColorType, Encoder};
use jpeg_decoder::Decoder;
use std::io::Cursor;

pub fn encode(
    data: &[u8],
    shape: &[usize],
    quality: u8,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (height, width, channels) = parse_image_shape(shape)?;

    let color_type = match channels {
        1 => EncColorType::Luma,
        3 => EncColorType::Rgb,
        4 => EncColorType::Rgba,
        _ => return Err(format!("unsupported channel count: {channels}").into()),
    };

    let mut output = Vec::new();
    let encoder = Encoder::new(&mut output, quality);
    encoder.encode(data, width as u16, height as u16, color_type)?;
    Ok(output)
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut decoder = Decoder::new(Cursor::new(data));
    let pixels = decoder.decode()?;
    Ok(pixels)
}

fn parse_image_shape(
    shape: &[usize],
) -> Result<(usize, usize, usize), Box<dyn std::error::Error + Send + Sync>> {
    match shape.len() {
        2 => Ok((shape[0], shape[1], 1)),
        3 => Ok((shape[0], shape[1], shape[2])),
        _ => Err(format!(
            "expected 2D or 3D array (HxW or HxWxC), got {} dimensions",
            shape.len()
        )
        .into()),
    }
}
