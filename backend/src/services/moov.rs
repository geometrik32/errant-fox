use anyhow::{anyhow, Result};

pub struct MoovInfo {
    pub fps: f32,
}

/// Parse the first bytes of an MP4 file to extract FPS from the moov atom.
/// Expects the moov atom to be at the beginning of the data (faststart / web-optimized).
/// Finds the VIDEO track (with vmhd) and extracts: mdhd.timescale / stts.sample_delta.
pub fn parse_fps(data: &[u8]) -> Result<MoovInfo> {
    let moov = find_atom(data, 0, b"moov")
        .ok_or_else(|| anyhow!("moov atom not found — video may not be faststart"))?;

    let trak = find_video_trak(data, moov)
        .ok_or_else(|| anyhow!("video trak (with vmhd) not found in moov"))?;

    let mdia = find_child_atom(data, trak, b"mdia")
        .ok_or_else(|| anyhow!("mdia atom not found in trak"))?;

    let mdhd_range = find_child_atom(data, mdia, b"mdhd")
        .ok_or_else(|| anyhow!("mdhd atom not found in mdia"))?;

    // mdhd box: timescale at byte 20 (8B header + 1B ver + 3B flags + 4B+4B times)
    let timescale = read_u32_be(data, mdhd_range.0 + 20);

    let minf = find_child_atom(data, mdia, b"minf")
        .ok_or_else(|| anyhow!("minf atom not found in mdia"))?;
    let stbl = find_child_atom(data, minf, b"stbl")
        .ok_or_else(|| anyhow!("stbl atom not found in minf"))?;
    let stts_range = find_child_atom(data, stbl, b"stts")
        .ok_or_else(|| anyhow!("stts atom not found in stbl"))?;

    // stts entries: [4B sample_count][4B sample_delta]...
    // first sample_delta at byte 20 (8B header + 1B ver + 3B flags + 4B count + 4B sample_count)
    let sample_delta = read_u32_be(data, stts_range.0 + 20);

    if sample_delta == 0 {
        return Err(anyhow!("stts sample_delta is zero — cannot compute fps"));
    }

    let fps = (timescale as f32 / sample_delta as f32).round();

    Ok(MoovInfo { fps })
}

// ── low-level helpers ──────────────────────────────────────────────────────────

type AtomRange = (usize, usize); // (start, end) — end is exclusive

fn read_u32_be(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn read_u64_be(data: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ])
}

fn atom_size(data: &[u8], offset: usize) -> (u64, usize) {
    let s = read_u32_be(data, offset) as u64;
    if s == 1 {
        (read_u64_be(data, offset + 8), 16)
    } else if s == 0 {
        ((data.len() - offset) as u64, 8)
    } else {
        (s, 8)
    }
}

fn find_atom(data: &[u8], start: usize, fourcc: &[u8; 4]) -> Option<AtomRange> {
    let mut offset = start;
    while offset + 8 <= data.len() {
        let (size, header_len) = atom_size(data, offset);
        if size < header_len as u64 {
            break;
        }
        let atom_type: &[u8; 4] = data[offset + 4..offset + 8].try_into().unwrap();
        let end = offset + size as usize;
        if atom_type == fourcc {
            return Some((offset, end.min(data.len())));
        }
        if end > data.len() {
            break;
        }
        offset = end;
    }
    None
}

fn find_child_atom(data: &[u8], parent: AtomRange, fourcc: &[u8; 4]) -> Option<AtomRange> {
    let header_len = if read_u32_be(data, parent.0) == 1 { 16 } else { 8 };
    find_atom(data, parent.0 + header_len, fourcc)
}

/// Check if a trak is a video track (has `vmhd` in minf or `hdlr` type `vide`).
fn is_video_trak(data: &[u8], trak: AtomRange) -> bool {
    if let Some(mdia) = find_child_atom(data, trak, b"mdia") {
        // Check hdlr for 'vide'
        if let Some(hdlr) = find_child_atom(data, mdia, b"hdlr") {
            // handler type is at byte 20 of hdlr
            if hdlr.0 + 20 + 4 <= data.len() {
                let htype: &[u8; 4] = data[hdlr.0 + 20..hdlr.0 + 24].try_into().unwrap();
                if htype == b"vide" {
                    return true;
                }
            }
        }
        // Check minf for vmhd
        if let Some(minf) = find_child_atom(data, mdia, b"minf") {
            if find_child_atom(data, minf, b"vmhd").is_some() {
                return true;
            }
        }
    }
    false
}

/// Find the video `trak` — the one that passes `is_video_trak`.
fn find_video_trak(data: &[u8], moov: AtomRange) -> Option<AtomRange> {
    let header_len = if read_u32_be(data, moov.0) == 1 { 16 } else { 8 };
    let mut offset = moov.0 + header_len;
    let limit = moov.1.min(data.len());

    while offset + 8 <= limit {
        let (size, hlen) = atom_size(data, offset);
        if size < hlen as u64 {
            break;
        }
        let atom_type: &[u8; 4] = data[offset + 4..offset + 8].try_into().unwrap();
        let end = offset + size as usize;

        if atom_type == b"trak" {
            let trak_range = (offset, end.min(limit));
            if is_video_trak(data, trak_range) {
                return Some(trak_range);
            }
        }

        if end > limit {
            break;
        }
        offset = end;
    }
    None
}
