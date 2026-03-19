use imgref::Img;
use ravif::RGBA8;

pub fn encode(
    data: &[u8],
    shape: &[usize],
    quality: Option<f32>,
    speed: Option<u8>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (height, width, channels) = parse_image_shape(shape)?;

    // Convert to RGBA
    let rgba_data: Vec<RGBA8> = match channels {
        3 => data
            .chunks_exact(3)
            .map(|c| RGBA8::new(c[0], c[1], c[2], 255))
            .collect(),
        4 => data
            .chunks_exact(4)
            .map(|c| RGBA8::new(c[0], c[1], c[2], c[3]))
            .collect(),
        _ => {
            return Err(
                format!("AVIF requires 3 or 4 channels, got {channels}").into(),
            )
        }
    };

    let img = Img::new(&rgba_data[..], width, height);
    let mut encoder = ravif::Encoder::new();

    if let Some(q) = quality {
        encoder = encoder.with_quality(q);
    }
    if let Some(s) = speed {
        encoder = encoder.with_speed(s);
    }

    let result = encoder.encode_rgba(img)?;
    Ok(result.avif_file)
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let decoder = avif_decode::Decoder::from_avif(data)?;
    let image = decoder.to_image()?;

    match image {
        avif_decode::Image::Rgb8(img) => {
            let pixels = img.buf();
            let mut out = Vec::with_capacity(pixels.len() * 3);
            for p in pixels {
                out.push(p.r);
                out.push(p.g);
                out.push(p.b);
            }
            Ok(out)
        }
        avif_decode::Image::Rgba8(img) => {
            let pixels = img.buf();
            let mut out = Vec::with_capacity(pixels.len() * 4);
            for p in pixels {
                out.push(p.r);
                out.push(p.g);
                out.push(p.b);
                out.push(p.a);
            }
            Ok(out)
        }
        avif_decode::Image::Rgb16(img) => {
            let pixels = img.buf();
            let mut out = Vec::with_capacity(pixels.len() * 3);
            for p in pixels {
                out.push((p.r >> 8) as u8);
                out.push((p.g >> 8) as u8);
                out.push((p.b >> 8) as u8);
            }
            Ok(out)
        }
        avif_decode::Image::Rgba16(img) => {
            let pixels = img.buf();
            let mut out = Vec::with_capacity(pixels.len() * 4);
            for p in pixels {
                out.push((p.r >> 8) as u8);
                out.push((p.g >> 8) as u8);
                out.push((p.b >> 8) as u8);
                out.push((p.a >> 8) as u8);
            }
            Ok(out)
        }
        avif_decode::Image::Gray8(img) => {
            Ok(img.buf().iter().map(|p| p.0).collect())
        }
        avif_decode::Image::Gray16(img) => {
            Ok(img.buf().iter().map(|p| (p.0 >> 8) as u8).collect())
        }
    }
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
