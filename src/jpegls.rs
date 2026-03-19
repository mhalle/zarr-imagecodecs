use charls::{CharLS, FrameInfo, InterleaveMode};

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
        component_count: components as i32,
    };

    if components == 1 {
        let mut codec = CharLS::default();
        return Ok(codec.encode(frame_info, near, data)?);
    }

    // Multi-component: use charls-sys directly with a generous buffer
    unsafe {
        let encoder = charls_sys::charls_jpegls_encoder_create();
        if encoder.is_null() {
            return Err("failed to create JPEG-LS encoder".into());
        }
        let _guard = EncoderGuard(encoder);

        let fi = charls_sys::charls_frame_info {
            width: width as u32,
            height: height as u32,
            bits_per_sample: 8,
            component_count: components as i32,
        };

        check_err(charls_sys::charls_jpegls_encoder_set_frame_info(
            encoder, &fi,
        ))?;

        check_err(charls_sys::charls_jpegls_encoder_set_interleave_mode(
            encoder,
            charls_sys::charls_interleave_mode_sample,
        ))?;

        check_err(charls_sys::charls_jpegls_encoder_set_near_lossless(
            encoder, near,
        ))?;

        let buf_size = data.len() * 2 + 1024;
        let mut dst = vec![0u8; buf_size];

        check_err(charls_sys::charls_jpegls_encoder_set_destination_buffer(
            encoder,
            dst.as_mut_ptr() as *mut std::ffi::c_void,
            buf_size,
        ))?;

        let mut src = data.to_vec();
        check_err(charls_sys::charls_jpegls_encoder_encode_from_buffer(
            encoder,
            src.as_mut_ptr() as *mut std::ffi::c_void,
            src.len(),
            0,
        ))?;

        let mut bytes_written: usize = 0;
        check_err(charls_sys::charls_jpegls_encoder_get_bytes_written(
            encoder,
            &mut bytes_written,
        ))?;

        dst.truncate(bytes_written);
        Ok(dst)
    }
}

pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Use charls-sys directly for all decode paths
    unsafe {
        let decoder = charls_sys::charls_jpegls_decoder_create();
        if decoder.is_null() {
            return Err("failed to create JPEG-LS decoder".into());
        }
        let _guard = DecoderGuard(decoder);

        check_err(charls_sys::charls_jpegls_decoder_set_source_buffer(
            decoder,
            data.as_ptr() as *const std::ffi::c_void,
            data.len(),
        ))?;

        check_err(charls_sys::charls_jpegls_decoder_read_header(decoder))?;

        // Get destination size
        let mut dest_size: usize = 0;
        check_err(
            charls_sys::charls_jpegls_decoder_get_destination_size(
                decoder,
                0, // stride
                &mut dest_size,
            ),
        )?;

        let mut dst = vec![0u8; dest_size];
        check_err(
            charls_sys::charls_jpegls_decoder_decode_to_buffer(
                decoder,
                dst.as_mut_ptr() as *mut std::ffi::c_void,
                dest_size,
                0, // stride
            ),
        )?;

        Ok(dst)
    }
}

// RAII guards
struct EncoderGuard(*mut charls_sys::charls_jpegls_encoder);
impl Drop for EncoderGuard {
    fn drop(&mut self) {
        unsafe { charls_sys::charls_jpegls_encoder_destroy(self.0); }
    }
}

struct DecoderGuard(*mut charls_sys::charls_jpegls_decoder);
impl Drop for DecoderGuard {
    fn drop(&mut self) {
        unsafe { charls_sys::charls_jpegls_decoder_destroy(self.0); }
    }
}

fn check_err(
    err: charls_sys::charls_jpegls_errc,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if err == 0 {
        Ok(())
    } else {
        Err(format!("JPEG-LS error code: {}", err).into())
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
