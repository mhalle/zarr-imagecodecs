/// DICOM RLE (Run Length Encoding) as defined in DICOM Transfer Syntax
/// 1.2.840.10008.1.2.5 (RLE Lossless).
///
/// Format:
/// - 64-byte RLE header: [num_segments(u32le), offset1..offset15(u32le)]
/// - Segments: one per byte plane, each PackBits-encoded
///
/// For N-byte-per-sample data with C components, there are N*C segments.
/// Byte planes are ordered: component 0 MSB, component 0 MSB-1, ...,
/// component 0 LSB, component 1 MSB, ..., component C-1 LSB.

use crate::packbits;

/// Maximum segments in DICOM RLE (header has room for 15 offsets).
const MAX_SEGMENTS: usize = 15;

pub fn encode(
    data: &[u8],
    width: usize,
    height: usize,
    samples_per_pixel: usize,
    bytes_per_sample: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let pixel_count = width * height;
    let num_segments = samples_per_pixel * bytes_per_sample;
    let expected_len = pixel_count * samples_per_pixel * bytes_per_sample;

    if num_segments > MAX_SEGMENTS {
        return Err(format!(
            "DICOM RLE supports at most {MAX_SEGMENTS} segments, got {num_segments}"
        )
        .into());
    }

    if data.len() < expected_len {
        return Err(format!(
            "buffer too small: got {} bytes, expected {} \
             ({}x{} pixels, {} samples, {} bytes/sample)",
            data.len(),
            expected_len,
            width,
            height,
            samples_per_pixel,
            bytes_per_sample,
        )
        .into());
    }

    // Decompose into byte planes and PackBits-encode each
    let mut segments: Vec<Vec<u8>> = Vec::with_capacity(num_segments);

    for component in 0..samples_per_pixel {
        for byte_idx in 0..bytes_per_sample {
            // Extract one byte plane: for each pixel, take the
            // (bytes_per_sample - 1 - byte_idx)th byte of this component
            // (MSB first ordering)
            let byte_offset = bytes_per_sample - 1 - byte_idx;
            let mut plane = Vec::with_capacity(pixel_count);

            for pixel in 0..pixel_count {
                let sample_start =
                    pixel * samples_per_pixel * bytes_per_sample
                        + component * bytes_per_sample;
                plane.push(data[sample_start + byte_offset]);
            }

            segments.push(packbits::encode(&plane));
        }
    }

    // Build RLE header (64 bytes = 16 u32le values)
    // First u32: number of segments
    // Next 15 u32s: byte offsets from start of header to each segment
    let mut header = [0u32; 16];
    header[0] = num_segments as u32;

    let mut offset = 64u32; // segments start after header
    for (i, seg) in segments.iter().enumerate() {
        header[i + 1] = offset;
        offset += seg.len() as u32;
    }

    // Write output
    let mut output = Vec::with_capacity(offset as usize);

    // Write header as little-endian u32s
    for val in &header {
        output.extend_from_slice(&val.to_le_bytes());
    }

    // Write segments
    for seg in &segments {
        output.extend_from_slice(seg);
    }

    // DICOM RLE segments must start on even byte boundaries
    // (each segment is padded to even length)
    // Actually, only the total frame needs to be even-length
    if output.len() % 2 != 0 {
        output.push(0);
    }

    Ok(output)
}

pub fn decode(
    data: &[u8],
    width: usize,
    height: usize,
    samples_per_pixel: usize,
    bytes_per_sample: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    if data.len() < 64 {
        return Err("DICOM RLE data too short for header".into());
    }

    // Parse header
    let num_segments = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let expected_segments = samples_per_pixel * bytes_per_sample;

    if num_segments != expected_segments {
        return Err(format!(
            "expected {expected_segments} segments, header says {num_segments}"
        )
        .into());
    }

    let mut offsets = Vec::with_capacity(num_segments);
    for i in 0..num_segments {
        let o = u32::from_le_bytes(
            data[(i + 1) * 4..(i + 2) * 4].try_into().unwrap(),
        ) as usize;

        if o < 64 {
            return Err(format!(
                "segment {i} offset {o} is inside the 64-byte header"
            )
            .into());
        }
        if o > data.len() {
            return Err(format!(
                "segment {i} offset {o} exceeds data length {}",
                data.len()
            )
            .into());
        }
        if let Some(&prev) = offsets.last() {
            if o < prev {
                return Err(format!(
                    "segment {i} offset {o} is before segment {} offset {prev}",
                    i - 1
                )
                .into());
            }
        }

        offsets.push(o);
    }

    // Decode each segment
    let pixel_count = width * height;
    let mut planes: Vec<Vec<u8>> = Vec::with_capacity(num_segments);

    for i in 0..num_segments {
        let start = offsets[i];
        let end = if i + 1 < num_segments {
            offsets[i + 1]
        } else {
            data.len()
        };

        let plane = packbits::decode_max(&data[start..end], pixel_count)?;
        if plane.len() < pixel_count {
            return Err(format!(
                "segment {i} decoded to {} bytes, expected {pixel_count}",
                plane.len()
            )
            .into());
        }
        planes.push(plane);
    }

    // Reassemble: interleave byte planes back into pixel data
    let total_bytes = pixel_count * samples_per_pixel * bytes_per_sample;
    let mut output = vec![0u8; total_bytes];

    let mut seg_idx = 0;
    for component in 0..samples_per_pixel {
        for byte_idx in 0..bytes_per_sample {
            let byte_offset = bytes_per_sample - 1 - byte_idx;
            let plane = &planes[seg_idx];

            for pixel in 0..pixel_count {
                let sample_start =
                    pixel * samples_per_pixel * bytes_per_sample
                        + component * bytes_per_sample;
                output[sample_start + byte_offset] = plane[pixel];
            }

            seg_idx += 1;
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_8bit_gray() {
        let data: Vec<u8> = (0..64).map(|i| (i * 4) as u8).collect();
        let enc = encode(&data, 8, 8, 1, 1).unwrap();
        let dec = decode(&enc, 8, 8, 1, 1).unwrap();
        assert_eq!(data, dec);
    }

    #[test]
    fn roundtrip_8bit_rgb() {
        let data: Vec<u8> = (0..192).map(|i| (i % 256) as u8).collect();
        let enc = encode(&data, 8, 8, 3, 1).unwrap();
        let dec = decode(&enc, 8, 8, 3, 1).unwrap();
        assert_eq!(data, dec);
    }

    #[test]
    fn roundtrip_16bit_gray() {
        // 16-bit grayscale: 2 bytes per sample
        let values: Vec<u16> = (0..64).map(|i| i * 1000).collect();
        let data: Vec<u8> = values
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let enc = encode(&data, 8, 8, 1, 2).unwrap();
        let dec = decode(&enc, 8, 8, 1, 2).unwrap();
        assert_eq!(data, dec);
    }

    #[test]
    fn header_structure() {
        let data = vec![42u8; 64]; // 8x8 grayscale
        let enc = encode(&data, 8, 8, 1, 1).unwrap();
        // Check header
        let num_seg = u32::from_le_bytes(enc[0..4].try_into().unwrap());
        assert_eq!(num_seg, 1);
        let offset1 = u32::from_le_bytes(enc[4..8].try_into().unwrap());
        assert_eq!(offset1, 64); // first segment starts at byte 64
    }
}
