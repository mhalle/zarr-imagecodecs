use std::io::{BufReader, Cursor};

pub fn encode(
    data: &[u8],
    shape: &[usize],
    _quality: Option<f32>,
    _lossless: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (height, width, channels) = parse_image_shape(shape)?;

    let color = match channels {
        3 => image_webp::ColorType::Rgb8,
        4 => image_webp::ColorType::Rgba8,
        _ => {
            return Err(
                format!("WebP requires 3 or 4 channels, got {channels}").into(),
            )
        }
    };

    // Note: image-webp only supports lossless VP8L encoding
    let mut output = Vec::new();
    let encoder = image_webp::WebPEncoder::new(&mut output);
    encoder.encode(data, width as u32, height as u32, color)?;
    Ok(output)
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let reader = BufReader::new(Cursor::new(data));
    let mut decoder = image_webp::WebPDecoder::new(reader)?;

    let buf_size = decoder
        .output_buffer_size()
        .ok_or("failed to get output buffer size")?;
    let mut buf = vec![0u8; buf_size];
    decoder.read_image(&mut buf)?;
    Ok(buf)
}

fn parse_image_shape(
    shape: &[usize],
) -> Result<(usize, usize, usize), Box<dyn std::error::Error + Send + Sync>> {
    match shape.len() {
        2 => Ok((shape[0], shape[1], 3)),
        3 => Ok((shape[0], shape[1], shape[2])),
        _ => Err(format!(
            "expected 2D or 3D array (HxW or HxWxC), got {} dimensions",
            shape.len()
        )
        .into()),
    }
}
