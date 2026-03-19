use charls::CharLS;
use charls::FrameInfo;

pub fn encode(
    data: &[u8],
    shape: &[usize],
    near: i32,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (height, width, components) = parse_image_shape(shape)?;

    let frame_info = FrameInfo {
        width: width as u32,
        height: height as u32,
        bits_per_sample: 8,
        component_count: 1,
    };

    if components == 1 {
        let mut codec = CharLS::default();
        return Ok(codec.encode(frame_info, near, data)?);
    }

    // Multi-component: encode each plane separately, then concatenate
    // with a header: [num_components(u8), len0(u32le), data0, len1(u32le), data1, ...]
    let plane_size = width * height;
    let mut output = Vec::new();
    output.push(components as u8);

    for c in 0..components {
        let plane_info = FrameInfo {
            width: width as u32,
            height: height as u32,
            bits_per_sample: 8,
            component_count: 1,
        };

        // Extract plane from interleaved data
        let plane: Vec<u8> = (0..plane_size)
            .map(|i| data[i * components + c])
            .collect();

        let mut codec = CharLS::default();
        let compressed = codec.encode(plane_info, near, &plane)?;

        // Write length as u32 LE, then data
        output.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        output.extend_from_slice(&compressed);
    }

    Ok(output)
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Check if this is our multi-plane format or a standard JPEG-LS stream
    // Standard JPEG-LS starts with FF D8 (SOI marker)
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
        // Standard single-component JPEG-LS
        let mut codec = CharLS::default();
        return Ok(codec.decode(data)?);
    }

    // Multi-plane format: [num_components(u8), len0(u32le), data0, ...]
    let components = data[0] as usize;
    let mut offset = 1;
    let mut planes: Vec<Vec<u8>> = Vec::new();

    for _ in 0..components {
        if offset + 4 > data.len() {
            return Err("truncated JPEG-LS multi-plane data".into());
        }
        let len = u32::from_le_bytes(
            data[offset..offset + 4].try_into().unwrap(),
        ) as usize;
        offset += 4;

        if offset + len > data.len() {
            return Err("truncated JPEG-LS plane data".into());
        }

        let mut codec = CharLS::default();
        let plane = codec.decode(&data[offset..offset + len])?;
        planes.push(plane);
        offset += len;
    }

    // Re-interleave planes
    let plane_size = planes[0].len();
    let mut interleaved = vec![0u8; plane_size * components];
    for c in 0..components {
        for i in 0..plane_size {
            interleaved[i * components + c] = planes[c][i];
        }
    }

    Ok(interleaved)
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
