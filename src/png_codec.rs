use png::{BitDepth, ColorType, Compression, Encoder as PngEncoder};
use std::io::Cursor;

pub fn encode(
    data: &[u8],
    shape: &[usize],
    level: Option<u8>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (height, width, channels) = parse_image_shape(shape)?;

    let color_type = match channels {
        1 => ColorType::Grayscale,
        2 => ColorType::GrayscaleAlpha,
        3 => ColorType::Rgb,
        4 => ColorType::Rgba,
        _ => return Err(format!("unsupported channel count: {channels}").into()),
    };

    let mut output = Vec::new();
    {
        let mut encoder =
            PngEncoder::new(&mut output, width as u32, height as u32);
        encoder.set_color(color_type);
        encoder.set_depth(BitDepth::Eight);

        if let Some(l) = level {
            let compression = match l {
                0 => Compression::Fast,
                _ => Compression::Best,
            };
            encoder.set_compression(compression);
        }

        let mut writer = encoder.write_header()?;
        writer.write_image_data(data)?;
    }
    Ok(output)
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let decoder = png::Decoder::new(Cursor::new(data));
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    buf.truncate(info.buffer_size());
    Ok(buf)
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
