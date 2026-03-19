use std::io::Cursor;
use zune_core::bit_depth::BitDepth;
use zune_core::colorspace::ColorSpace;
use zune_core::options::EncoderOptions;
use zune_jpegxl::JxlSimpleEncoder;

pub fn encode(
    data: &[u8],
    shape: &[usize],
    effort: Option<u8>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (height, width, channels) = parse_image_shape(shape)?;

    let colorspace = match channels {
        1 => ColorSpace::Luma,
        3 => ColorSpace::RGB,
        4 => ColorSpace::RGBA,
        _ => return Err(format!("unsupported channel count: {channels}").into()),
    };

    let mut options = EncoderOptions::new(width, height, colorspace, BitDepth::Eight);

    if let Some(e) = effort {
        options = options.set_effort(e);
    }

    let encoder = JxlSimpleEncoder::new(data, options);
    let mut output = Vec::new();
    encoder
        .encode(&mut output)
        .map_err(|e| format!("JXL encode failed: {e:?}"))?;
    Ok(output)
}

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
