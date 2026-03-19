/// PackBits encode: raw bytes -> compressed bytes.
///
/// Standard TIFF/Apple PackBits RLE algorithm, also used for
/// DICOM RLE transfer syntax (1.2.840.10008.1.2.5).
pub fn encode(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let len = data.len();
    let mut i = 0;

    while i < len {
        // Count run of identical bytes (max 128)
        let mut run = 1;
        while i + run < len && run < 128 && data[i + run] == data[i] {
            run += 1;
        }

        if run >= 3 {
            // RLE run: header = -(run-1), then the repeated byte
            output.push((-(run as i8 - 1)) as u8);
            output.push(data[i]);
            i += run;
        } else {
            // Literal run (max 128 bytes)
            let start = i;
            let mut lit_len = 0;
            while i + lit_len < len && lit_len < 128 {
                // Look ahead for a run of 3+
                let mut ahead_run = 1;
                while i + lit_len + ahead_run < len
                    && ahead_run < 128
                    && data[i + lit_len + ahead_run] == data[i + lit_len]
                {
                    ahead_run += 1;
                }
                if ahead_run >= 3 {
                    break;
                }
                lit_len += 1;
            }
            if lit_len == 0 {
                lit_len = 1;
            }
            // Header = lit_len - 1, then the literal bytes
            output.push((lit_len - 1) as u8);
            output.extend_from_slice(&data[start..start + lit_len]);
            i = start + lit_len;
        }
    }

    output
}

/// PackBits decode: compressed bytes -> raw bytes.
pub fn decode(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    decode_max(data, usize::MAX)
}

/// PackBits decode with a maximum output size.
/// Stops after producing `max_output` bytes, ignoring any trailing data.
pub fn decode_max(
    data: &[u8],
    max_output: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut output = Vec::new();
    let mut i = 0;

    while i < data.len() && output.len() < max_output {
        let header = data[i] as i8;
        i += 1;

        if header >= 0 {
            // Literal: copy next (header + 1) bytes
            let count = header as usize + 1;
            if i + count > data.len() {
                return Err("packbits: unexpected end of data in literal run".into());
            }
            output.extend_from_slice(&data[i..i + count]);
            i += count;
        } else if header == -128 {
            // No-op
        } else {
            // RLE: repeat next byte (-header + 1) times
            let count = (-header) as usize + 1;
            if i >= data.len() {
                return Err("packbits: unexpected end of data in RLE run".into());
            }
            let byte = data[i];
            i += 1;
            output.resize(output.len() + count, byte);
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let data = b"AAAAAABBCCCCCCDDDDDDDDDD";
        let compressed = encode(data);
        let decompressed = decode(&compressed).unwrap();
        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn roundtrip_random() {
        let data: Vec<u8> = (0..1000).map(|i| (i * 37 % 256) as u8).collect();
        let compressed = encode(&data);
        let decompressed = decode(&compressed).unwrap();
        assert_eq!(data, decompressed);
    }
}
