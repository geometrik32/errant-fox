/**
 * Parse the moov atom from an MP4 file header to extract video FPS.
 * Fetches only the first 2 MB of the video via HTTP Range request.
 *
 * Navigates: moov → trak (video, with vmhd or 'vide' hdlr in mdia/minf)
 *   → mdia → mdhd (timescale) + stts (sample_delta)
 * FPS = timescale / sample_delta
 */

interface AtomRange {
  start: number;
  end: number; // exclusive
}

function readU32BE(data: Uint8Array, offset: number): number {
  return (
    (data[offset] << 24) |
    (data[offset + 1] << 16) |
    (data[offset + 2] << 8) |
    data[offset + 3]
  ) >>> 0;
}

function readU64BE(data: Uint8Array, offset: number): bigint {
  const hi = readU32BE(data, offset);
  const lo = readU32BE(data, offset + 4);
  return (BigInt(hi) << 32n) | BigInt(lo);
}

function atomSize(data: Uint8Array, offset: number): { size: bigint; headerLen: number } {
  const s = BigInt(readU32BE(data, offset));
  if (s === 1n) {
    return { size: readU64BE(data, offset + 8), headerLen: 16 };
  } else if (s === 0n) {
    return { size: BigInt(data.length - offset), headerLen: 8 };
  } else {
    return { size: s, headerLen: 8 };
  }
}

function fourccAt(data: Uint8Array, offset: number): string {
  return String.fromCharCode(
    data[offset],
    data[offset + 1],
    data[offset + 2],
    data[offset + 3],
  );
}

function findAtom(data: Uint8Array, start: number, fourcc: string): AtomRange | null {
  let offset = start;
  while (offset + 8 <= data.length) {
    const { size, headerLen } = atomSize(data, offset);
    if (size < BigInt(headerLen)) break;
    const atomType = fourccAt(data, offset + 4);
    const end = offset + Number(size);
    if (atomType === fourcc) {
      return { start: offset, end: Math.min(end, data.length) };
    }
    if (end > data.length) break;
    offset = end;
  }
  return null;
}

function findChildAtom(
  data: Uint8Array,
  parent: AtomRange,
  fourcc: string,
): AtomRange | null {
  const headerLen = readU32BE(data, parent.start) === 1 ? 16 : 8;
  return findAtom(data, parent.start + headerLen, fourcc);
}

/// Iterate all direct children of `parent` matching `fourcc`.
function* iterChildAtoms(
  data: Uint8Array,
  parent: AtomRange,
  fourcc: string,
): Generator<AtomRange> {
  const headerLen = readU32BE(data, parent.start) === 1 ? 16 : 8;
  let offset = parent.start + headerLen;
  const limit = Math.min(parent.end, data.length);
  while (offset + 8 <= limit) {
    const { size } = atomSize(data, offset);
    if (size < 8n) break;
    const atomType = fourccAt(data, offset + 4);
    const end = offset + Number(size);
    const clampedEnd = Math.min(end, limit);
    if (atomType === fourcc) {
      yield { start: offset, end: clampedEnd };
    }
    if (end > limit) break;
    offset = end;
  }
}

/// Check if a trak is a video track.
/// A video track has either `vmhd` in `minf`, or `hdlr` with handler type `vide`.
function isVideoTrak(data: Uint8Array, trak: AtomRange): boolean {
  const mdia = findChildAtom(data, trak, 'mdia');
  if (!mdia) return false;

  // Check hdlr for 'vide' (handler type at offset 20 = 8B header + 1B ver + 3B flags + 4B type + 4B subtype)
  const hdlr = findChildAtom(data, mdia, 'hdlr');
  if (hdlr) {
    const handlerType = fourccAt(data, hdlr.start + 20);
    if (handlerType === 'vide') return true;
  }

  // Check minf for vmhd
  const minf = findChildAtom(data, mdia, 'minf');
  if (minf && findChildAtom(data, minf, 'vmhd')) return true;

  return false;
}

export async function extractFpsFromUrl(videoSrc: string): Promise<number | null> {
  try {
    const resp = await fetch(videoSrc, {
      headers: { Range: 'bytes=0-2097151' }, // first 2 MB
    });
    if (!resp.ok) {
      console.warn('[moov] fetch failed:', resp.status, resp.statusText);
      return null;
    }

    const buf = await resp.arrayBuffer();
    const data = new Uint8Array(buf);
    console.log('[moov] fetched', data.length, 'bytes, first fourcc:', fourccAt(data, 4));

    const moov = findAtom(data, 0, 'moov');
    if (!moov) {
      console.warn('[moov] moov atom not found in first', data.length, 'bytes');
      return null;
    }
    console.log('[moov] found moov at', moov.start, 'size', moov.end - moov.start);

    // Find video trak
    let videoTrak: AtomRange | null = null;
    for (const trak of iterChildAtoms(data, moov, 'trak')) {
      if (isVideoTrak(data, trak)) {
        videoTrak = trak;
        break;
      }
    }
    if (!videoTrak) {
      console.warn('[moov] no video trak found');
      return null;
    }

    const mdia = findChildAtom(data, videoTrak, 'mdia');
    if (!mdia) { console.warn('[moov] mdia not found'); return null; }

    const mdhd = findChildAtom(data, mdia, 'mdhd');
    if (!mdhd) { console.warn('[moov] mdhd not found'); return null; }

    const timescale = readU32BE(data, mdhd.start + 20);

    const minf = findChildAtom(data, mdia, 'minf');
    if (!minf) { console.warn('[moov] minf not found'); return null; }
    const stbl = findChildAtom(data, minf, 'stbl');
    if (!stbl) { console.warn('[moov] stbl not found'); return null; }
    const stts = findChildAtom(data, stbl, 'stts');
    if (!stts) { console.warn('[moov] stts not found'); return null; }

    // stts entries: [4B sample_count][4B sample_delta]...
    // first sample_delta is at byte 20 (8B header + 1B ver + 3B flags + 4B count + 4B sample_count)
    const sampleDelta = readU32BE(data, stts.start + 20);
    console.log('[moov] timescale:', timescale, 'sampleDelta:', sampleDelta);

    if (sampleDelta === 0 || timescale === 0) {
      console.warn('[moov] zero timescale or sampleDelta');
      return null;
    }

    const fps = Math.round(timescale / sampleDelta);
    console.log('[moov] detected fps:', fps);
    return fps;
  } catch (e) {
    console.warn('[moov] exception:', e);
    return null;
  }
}
