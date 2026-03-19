use std::io::Cursor;
use tiff::decoder::Decoder as TiffDecoder;
use tiff::encoder::{colortype, TiffEncoder};

pub fn encode(
    data: &[u8],
    shape: &[usize],
    _compression: Option<&str>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (height, width, channels) = parse_image_shape(shape)?;

    let mut buf = Cursor::new(Vec::new());
    let mut encoder = TiffEncoder::new(&mut buf)?;

    match channels {
        1 => encoder.write_image::<colortype::Gray8>(width as u32, height as u32, data)?,
        3 => encoder.write_image::<colortype::RGB8>(width as u32, height as u32, data)?,
        4 => encoder.write_image::<colortype::RGBA8>(width as u32, height as u32, data)?,
        _ => {
            return Err(
                format!("unsupported channel count for TIFF: {channels}").into(),
            )
        }
    }

    Ok(buf.into_inner())
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let cursor = Cursor::new(data);
    let mut decoder = TiffDecoder::new(cursor)?;
    let result = decoder.read_image()?;

    match result {
        tiff::decoder::DecodingResult::U8(buf) => Ok(buf),
        tiff::decoder::DecodingResult::U16(buf) => {
            Ok(buf.iter().map(|&v| (v >> 8) as u8).collect())
        }
        _ => Err("unsupported TIFF pixel format".into()),
    }
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
