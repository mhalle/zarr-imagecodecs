use std::ffi::c_void;
use std::os::raw::c_int;

extern "C" {
    fn ojph_encode(
        pixels: *const u8,
        width: u32,
        height: u32,
        num_comps: u32,
        bit_depth: u32,
        is_signed: c_int,
        reversible: c_int,
        quant_step: f32,
        num_decomps: u32,
        out_buf: *mut *mut u8,
        out_size: *mut usize,
    ) -> c_int;

    fn ojph_decode(
        data: *const u8,
        data_size: usize,
        out_buf: *mut *mut u8,
        out_size: *mut usize,
        out_width: *mut u32,
        out_height: *mut u32,
        out_comps: *mut u32,
        out_depth: *mut u32,
        out_signed: *mut c_int,
    ) -> c_int;

    fn ojph_free(buf: *mut u8);
}

pub fn encode(
    data: &[u8],
    shape: &[usize],
    reversible: bool,
    quant_step: Option<f32>,
    num_decomps: Option<u32>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let (height, width, components) = parse_image_shape(shape)?;

    let mut out_buf: *mut u8 = std::ptr::null_mut();
    let mut out_size: usize = 0;

    let ret = unsafe {
        ojph_encode(
            data.as_ptr(),
            width as u32,
            height as u32,
            components as u32,
            8, // bit_depth
            0, // not signed
            if reversible { 1 } else { 0 },
            quant_step.unwrap_or(0.0),
            num_decomps.unwrap_or(5),
            &mut out_buf,
            &mut out_size,
        )
    };

    if ret != 0 || out_buf.is_null() {
        return Err("HTJ2K encode failed".into());
    }

    let result = unsafe { std::slice::from_raw_parts(out_buf, out_size).to_vec() };
    unsafe { ojph_free(out_buf) };
    Ok(result)
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut out_buf: *mut u8 = std::ptr::null_mut();
    let mut out_size: usize = 0;
    let mut out_width: u32 = 0;
    let mut out_height: u32 = 0;
    let mut out_comps: u32 = 0;
    let mut out_depth: u32 = 0;
    let mut out_signed: c_int = 0;

    let ret = unsafe {
        ojph_decode(
            data.as_ptr(),
            data.len(),
            &mut out_buf,
            &mut out_size,
            &mut out_width,
            &mut out_height,
            &mut out_comps,
            &mut out_depth,
            &mut out_signed,
        )
    };

    if ret != 0 || out_buf.is_null() {
        return Err("HTJ2K decode failed".into());
    }

    let result = unsafe { std::slice::from_raw_parts(out_buf, out_size).to_vec() };
    unsafe { ojph_free(out_buf) };
    Ok(result)
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
