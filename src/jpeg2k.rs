pub fn encode(
    data: &[u8],
    shape: &[usize],
    _level: Option<f32>,
    _reversible: bool,
    _num_resolutions: Option<u32>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (height, width, components) = parse_image_shape(shape)?;

    let mut params = openjp2::opj_cparameters::default();

    if _reversible {
        params.irreversible = 0;
    } else {
        params.irreversible = 1;
    }

    params.tcp_numlayers = 1;
    params.cp_disto_alloc = 1;

    if let Some(q) = _level {
        params.tcp_rates[0] = q;
    }

    if let Some(nr) = _num_resolutions {
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

    // Fill component data (interleaved -> planar)
    for c in 0..num_comps as usize {
        let comp = unsafe { &mut *image.comps.add(c) };
        if let Some(comp_data) = comp.data_mut() {
            for i in 0..(width * height) {
                comp_data[i] = data[i * num_comps as usize + c] as i32;
            }
        }
    }

    // Write to temp file since Stream requires file paths
    let tmp_path = std::env::temp_dir().join(format!(
        "zarr_j2k_enc_{}.j2k",
        std::process::id()
    ));

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
    // Write to temp file since Stream requires file paths
    let tmp_path = std::env::temp_dir().join(format!(
        "zarr_j2k_dec_{}.j2k",
        std::process::id()
    ));
    std::fs::write(&tmp_path, data)?;

    let mut stream =
        openjp2::Stream::new_file(&tmp_path, 1024 * 1024, true)?;

    let mut codec =
        openjp2::Codec::new_decoder(openjp2::CODEC_FORMAT::OPJ_CODEC_J2K)
            .ok_or("failed to create decoder")?;

    let mut params = openjp2::opj_dparameters::default();
    codec.setup_decoder(&mut params);

    let mut image = codec
        .read_header(&mut stream)
        .ok_or("failed to read header")?;

    codec.decode(&mut stream, &mut image);
    codec.end_decompress(&mut stream);
    drop(stream);
    let _ = std::fs::remove_file(&tmp_path);

    let width = (image.x1 - image.x0) as usize;
    let height = (image.y1 - image.y0) as usize;
    let num_comps = image.numcomps as usize;

    // Planar -> interleaved
    let mut pixels = vec![0u8; width * height * num_comps];
    for c in 0..num_comps {
        let comp = unsafe { &*image.comps.add(c) };
        if let Some(comp_data) = comp.data() {
            for i in 0..(width * height) {
                pixels[i * num_comps + c] = comp_data[i].clamp(0, 255) as u8;
            }
        }
    }

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
