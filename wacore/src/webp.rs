//! WebP format utilities.

/// Detects animated WebP by parsing RIFF/VP8X headers.
pub fn is_animated(data: &[u8]) -> bool {
    // Minimum: RIFF(4) + size(4) + WEBP(4) + chunk header(8) = 20
    if data.len() < 20 {
        return false;
    }
    if &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
        return false;
    }

    let mut offset = 12;
    while offset + 8 <= data.len() {
        let fourcc = &data[offset..offset + 4];
        let chunk_size = u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;

        if fourcc == b"VP8X"
            && chunk_size >= 10
            && offset + 8 < data.len()
            && data[offset + 8] & 0x02 != 0
        {
            return true;
        }

        if fourcc == b"ANIM" || fourcc == b"ANMF" {
            return true;
        }

        // Each addition checked to prevent overflow on 32-bit
        offset = match offset
            .checked_add(8)
            .and_then(|v| v.checked_add(chunk_size))
            .and_then(|v| v.checked_add(chunk_size & 1))
        {
            Some(next) => next,
            None => break,
        };
    }

    false
}

/// Reads the pixel dimensions from a WebP header, whichever flavor it is.
///
/// - `VP8X` (extended): 24-bit canvas width/height, stored minus one.
/// - `VP8 ` (lossy): 14-bit width/height in the keyframe header.
/// - `VP8L` (lossless): 14-bit width/height, stored minus one, packed after
///   the signature byte.
///
/// Returns `None` for anything that is not a parseable WebP.
pub fn dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 20 || &data[0..4] != b"RIFF" || &data[8..12] != b"WEBP" {
        return None;
    }

    let mut offset = 12;
    while offset + 8 <= data.len() {
        let fourcc = &data[offset..offset + 4];
        let chunk_size = u32::from_le_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;
        let payload = data.get(offset + 8..)?;
        let payload = &payload[..chunk_size.min(payload.len())];

        match fourcc {
            b"VP8X" if payload.len() >= 10 => {
                let width = 1 + u32::from_le_bytes([payload[4], payload[5], payload[6], 0]);
                let height = 1 + u32::from_le_bytes([payload[7], payload[8], payload[9], 0]);
                return Some((width, height));
            }
            b"VP8 " if payload.len() >= 10 => {
                if payload[3..6] != [0x9D, 0x01, 0x2A] {
                    return None;
                }
                let width = u32::from(u16::from_le_bytes([payload[6], payload[7]]) & 0x3FFF);
                let height = u32::from(u16::from_le_bytes([payload[8], payload[9]]) & 0x3FFF);
                return Some((width, height));
            }
            b"VP8L" if payload.len() >= 5 => {
                if payload[0] != 0x2F {
                    return None;
                }
                let bits = u32::from_le_bytes([payload[1], payload[2], payload[3], payload[4]]);
                let width = 1 + (bits & 0x3FFF);
                let height = 1 + ((bits >> 14) & 0x3FFF);
                return Some((width, height));
            }
            _ => {}
        }

        // Each addition checked to prevent overflow on 32-bit
        offset = match offset
            .checked_add(8)
            .and_then(|v| v.checked_add(chunk_size))
            .and_then(|v| v.checked_add(chunk_size & 1))
        {
            Some(next) => next,
            None => break,
        };
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_webp_vp8x(flags: u8) -> Vec<u8> {
        let mut buf = Vec::new();
        // RIFF header
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&0u32.to_le_bytes()); // placeholder file size
        buf.extend_from_slice(b"WEBP");
        // VP8X chunk
        buf.extend_from_slice(b"VP8X");
        buf.extend_from_slice(&10u32.to_le_bytes()); // chunk size
        buf.push(flags);
        buf.extend_from_slice(&[0u8; 9]); // rest of VP8X payload
        // Fix RIFF size
        let riff_size = (buf.len() - 8) as u32;
        buf[4..8].copy_from_slice(&riff_size.to_le_bytes());
        buf
    }

    #[test]
    fn static_webp() {
        let data = make_webp_vp8x(0x00);
        assert!(!is_animated(&data));
    }

    #[test]
    fn animated_webp_via_flag() {
        let data = make_webp_vp8x(0x02);
        assert!(is_animated(&data));
    }

    #[test]
    fn animated_webp_via_anim_chunk() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"WEBP");
        // VP8X without animation flag
        buf.extend_from_slice(b"VP8X");
        buf.extend_from_slice(&10u32.to_le_bytes());
        buf.extend_from_slice(&[0u8; 10]);
        // ANIM chunk
        buf.extend_from_slice(b"ANIM");
        buf.extend_from_slice(&6u32.to_le_bytes());
        buf.extend_from_slice(&[0u8; 6]);
        let riff_size = (buf.len() - 8) as u32;
        buf[4..8].copy_from_slice(&riff_size.to_le_bytes());

        assert!(is_animated(&buf));
    }

    #[test]
    fn too_short() {
        assert!(!is_animated(&[0; 10]));
    }

    #[test]
    fn not_webp() {
        assert!(!is_animated(b"NOT A WEBP FILE AT ALL!!"));
    }

    fn make_webp_vp8x_sized(width: u32, height: u32) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"WEBP");
        buf.extend_from_slice(b"VP8X");
        buf.extend_from_slice(&10u32.to_le_bytes());
        buf.extend_from_slice(&[0u8; 4]);
        buf.extend_from_slice(&(width - 1).to_le_bytes()[..3]);
        buf.extend_from_slice(&(height - 1).to_le_bytes()[..3]);
        let riff_size = (buf.len() - 8) as u32;
        buf[4..8].copy_from_slice(&riff_size.to_le_bytes());
        buf
    }

    #[test]
    fn dimensions_vp8x() {
        assert_eq!(
            dimensions(&make_webp_vp8x_sized(512, 512)),
            Some((512, 512))
        );
        assert_eq!(
            dimensions(&make_webp_vp8x_sized(1, 16384)),
            Some((1, 16384))
        );
    }

    #[test]
    fn dimensions_vp8_lossy() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"WEBP");
        buf.extend_from_slice(b"VP8 ");
        buf.extend_from_slice(&10u32.to_le_bytes());
        buf.extend_from_slice(&[0u8; 3]);
        buf.extend_from_slice(&[0x9D, 0x01, 0x2A]);
        buf.extend_from_slice(&512u16.to_le_bytes());
        buf.extend_from_slice(&300u16.to_le_bytes());
        let riff_size = (buf.len() - 8) as u32;
        buf[4..8].copy_from_slice(&riff_size.to_le_bytes());

        assert_eq!(dimensions(&buf), Some((512, 300)));
    }

    #[test]
    fn dimensions_vp8_lossy_bad_start_code() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"WEBP");
        buf.extend_from_slice(b"VP8 ");
        buf.extend_from_slice(&10u32.to_le_bytes());
        buf.extend_from_slice(&[0u8; 10]);
        let riff_size = (buf.len() - 8) as u32;
        buf[4..8].copy_from_slice(&riff_size.to_le_bytes());

        assert_eq!(dimensions(&buf), None);
    }

    #[test]
    fn dimensions_vp8l() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(b"WEBP");
        buf.extend_from_slice(b"VP8L");
        buf.extend_from_slice(&5u32.to_le_bytes());
        buf.push(0x2F);
        let bits: u32 = (512 - 1) | ((512 - 1) << 14);
        buf.extend_from_slice(&bits.to_le_bytes());
        let riff_size = (buf.len() - 8) as u32;
        buf[4..8].copy_from_slice(&riff_size.to_le_bytes());

        assert_eq!(dimensions(&buf), Some((512, 512)));
    }

    #[test]
    fn dimensions_rejects_garbage() {
        assert_eq!(dimensions(&[0; 10]), None);
        assert_eq!(dimensions(b"NOT A WEBP FILE AT ALL!!"), None);
        let truncated = &make_webp_vp8x_sized(512, 512)[..16];
        assert_eq!(dimensions(truncated), None);
    }
}
