use jpeg2k::ImagePixelData;
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique temp file path (thread-safe).
fn temp_path(ext: &str) -> std::path::PathBuf {
    let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zarr_j2k_{}_{}.{}",
        std::process::id(),
        id,
        ext
    ))
}

pub fn encode(
    data: &[u8],
    shape: &[usize],
    level: Option<f32>,
    reversible: bool,
    num_resolutions: Option<u32>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (height, width, components) = parse_image_shape(shape)?;

    let mut params = openjp2::opj_cparameters::default();

    params.irreversible = if reversible { 0 } else { 1 };
    params.tcp_numlayers = 1;
    params.cp_disto_alloc = 1;

    if let Some(q) = level {
        params.tcp_rates[0] = q;
    }

    if let Some(nr) = num_resolutions {
        params.numresolution = nr as i32;
    }

    let num_comps = components as u32;
    let mut cmptparms = vec![
        openjp2::opj_image_comptparm {
            dx: 1,
            dy: 1,
            w: width as u32,
            h: height as u32,
            x0: 0,
            y0: 0,
            prec: 8,
            bpp: 8,
            sgnd: 0,
        };
        num_comps as usize
    ];

    let color_space = if num_comps >= 3 {
        openjp2::COLOR_SPACE::OPJ_CLRSPC_SRGB
    } else {
        openjp2::COLOR_SPACE::OPJ_CLRSPC_GRAY
    };

    let mut image_opt =
        openjp2::opj_image::create(&mut cmptparms, color_space);
    let image = image_opt
        .as_mut()
        .ok_or("failed to create opj_image")?;

    image.x0 = 0;
    image.y0 = 0;
    image.x1 = width as u32;
    image.y1 = height as u32;

    // Interleaved -> planar component data
    for c in 0..num_comps as usize {
        let comp = unsafe { &mut *image.comps.add(c) };
        if let Some(comp_data) = comp.data_mut() {
            for i in 0..(width * height) {
                comp_data[i] = data[i * num_comps as usize + c] as i32;
            }
        }
    }

    // Encode via temp file (openjp2 Stream requires file paths)
    let tmp_path = temp_path("j2k");

    let mut stream =
        openjp2::Stream::new_file(&tmp_path, 1024 * 1024, false)?;
    let mut codec =
        openjp2::Codec::new_encoder(openjp2::CODEC_FORMAT::OPJ_CODEC_J2K)
            .ok_or("failed to create encoder")?;

    codec.setup_encoder(&mut params, image);
    codec.start_compress(image, &mut stream);
    codec.encode(&mut stream);
    codec.end_compress(&mut stream);
    drop(stream);

    let result = std::fs::read(&tmp_path)?;
    let _ = std::fs::remove_file(&tmp_path);
    Ok(result)
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Use jpeg2k crate for clean in-memory decode
    let image = jpeg2k::Image::from_bytes(data)
        .map_err(|e| format!("JPEG 2000 decode failed: {e}"))?;

    let img_data = image
        .get_pixels(None)
        .map_err(|e| format!("JPEG 2000 pixel extraction failed: {e}"))?;

    match img_data.data {
        ImagePixelData::L8(buf) => Ok(buf),
        ImagePixelData::La8(buf) => Ok(buf),
        ImagePixelData::Rgb8(buf) => Ok(buf),
        ImagePixelData::Rgba8(buf) => Ok(buf),
        ImagePixelData::L16(buf) => {
            Ok(buf.iter().map(|&v| (v >> 8) as u8).collect())
        }
        ImagePixelData::Rgb16(buf) => {
            Ok(buf.iter().map(|&v| (v >> 8) as u8).collect())
        }
        _ => Err("unsupported JPEG 2000 pixel format".into()),
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
