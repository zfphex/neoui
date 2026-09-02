use crate::*;

#[cfg(feature = "jpeg")]
use zune_core::colorspace::ColorSpace;
#[cfg(any(feature = "jpeg", feature = "png"))]
use zune_core::{bytestream::ZCursor, options::DecoderOptions};
#[cfg(feature = "jpeg")]
use zune_jpeg::JpegDecoder;
#[cfg(feature = "png")]
use zune_png::PngDecoder;

const LANES: u32 = 0x00FF_00FF;
const BIAS: u32 = 0x0080_0080;
const RGB: u32 = 0x00FF_FFFF;
const WEIGHT_ONE: u32 = 256;

#[derive(Debug, Clone, Copy)]
pub struct Image<'a> {
    pub width: usize,
    pub height: usize,
    pub pixels: &'a [u32],
}

impl<'a> Image<'a> {
    pub fn new(width: usize, height: usize, pixels: &'a [u32]) -> Self {
        assert_eq!(pixels.len(), width * height);
        Self { width, height, pixels }
    }
}

impl std::hash::Hash for Image<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pixels.as_ptr().hash(state);
        self.pixels.len().hash(state);
        self.width.hash(state);
    }
}

#[inline]
pub fn mul(v: u8, a: u8) -> u32 {
    (v as u32 * a as u32 + 127) / 255
}

#[inline]
pub fn premultiply(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (a as u32) << 24 | mul(r, a) << 16 | mul(g, a) << 8 | mul(b, a)
}

#[cfg(any(feature = "jpeg", feature = "png"))]
pub fn decode(bytes: &[u8]) -> Result<(Vec<u32>, usize, usize), Box<dyn std::error::Error>> {
    let mut out: Vec<u32> = Vec::new();

    let (width, height, channels) = if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        #[cfg(not(feature = "png"))]
        return Err("png support is not enabled".into());
        #[cfg(feature = "png")]
        {
            let options = DecoderOptions::default()
                .png_set_add_alpha_channel(true)
                .png_set_strip_to_8bit(true);
            let mut decoder = PngDecoder::new_with_options(ZCursor::new(bytes), options);
            decoder.decode_headers()?;
            let (width, height) = decoder.dimensions().unwrap();
            let channels = decoder.output_buffer_size().unwrap() / (width * height);
            out.resize(width * height, 0);
            decoder
                .decode_into(unsafe { std::slice::from_raw_parts_mut(out.as_mut_ptr() as *mut u8, out.len() * 4) })?;
            (width, height, channels)
        }
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        #[cfg(not(feature = "jpeg"))]
        return Err("jpeg support is not enabled".into());
        #[cfg(feature = "jpeg")]
        {
            let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
            let mut decoder = JpegDecoder::new_with_options(ZCursor::new(bytes), options);
            decoder.decode_headers()?;
            let info = decoder.info().unwrap();
            let (width, height) = (info.width as usize, info.height as usize);
            let channels = decoder.output_buffer_size().unwrap() / (width * height);
            out.resize(width * height, 0);
            decoder
                .decode_into(unsafe { std::slice::from_raw_parts_mut(out.as_mut_ptr() as *mut u8, out.len() * 4) })?;
            (width, height, channels)
        }
    } else {
        return Err("not a png or jpeg".into());
    };

    match channels {
        4 => {
            for pixel in out.iter_mut() {
                let [r, g, b, a] = pixel.to_ne_bytes();
                *pixel = premultiply(r, g, b, a);
            }
        }
        2 => {
            for i in (0..width * height).rev() {
                let word = out[i / 2].to_ne_bytes();
                let (luma, alpha) = if i % 2 == 0 { (word[0], word[1]) } else { (word[2], word[3]) };
                out[i] = premultiply(luma, luma, luma, alpha);
            }
        }
        n => return Err(format!("unsupported channel count {n}"))?,
    }

    Ok((out, width, height))
}

pub fn resize(src: Image, width: usize, height: usize) -> Vec<u32> {
    assert!(width > 0 && height > 0 && src.width > 0 && src.height > 0);

    struct Axis {
        pub taps: usize,
        pub first: Vec<usize>,
        pub weights: Vec<u16>,
    }

    fn axis(source_len: usize, dest_len: usize) -> Axis {
        let ratio = source_len as f32 / dest_len as f32;
        let half = (ratio * 0.5).max(0.5);
        let taps = ((2.0 * half).ceil() as usize + 1).clamp(1, source_len);
        let last_window = source_len - taps;

        let mut first = Vec::with_capacity(dest_len);
        let mut weights = vec![0u16; dest_len * taps];
        let mut scratch = vec![0f32; taps];

        for i in 0..dest_len {
            let centre = (i as f32 + 0.5) * ratio;
            let (low, high) = (centre - half, centre + half);
            let begin = low.floor() as i64;
            let window = begin.clamp(0, last_window as i64) as usize;

            scratch.fill(0.0);
            let mut total = 0.0;
            for tap in 0..taps {
                let edge = begin + tap as i64;
                let weight = (((edge + 1) as f32).min(high) - (edge as f32).max(low)).max(0.0);
                let j = edge.clamp(0, source_len as i64 - 1) as usize;
                scratch[j.saturating_sub(window).min(taps - 1)] += weight;
                total += weight;
            }
            if total <= 0.0 {
                scratch[0] = 1.0;
                total = 1.0;
            }

            let normalize = WEIGHT_ONE as f32 / total;
            let (mut exact, mut assigned) = (0.0f32, 0u32);
            for tap in 0..taps {
                exact += scratch[tap] * normalize;
                let weight = (exact.round() as u32).saturating_sub(assigned).min(WEIGHT_ONE);
                weights[i * taps + tap] = weight as u16;
                assigned += weight;
            }
            first.push(window);
        }

        Axis { taps, first, weights }
    }

    #[inline(always)]
    fn gather(source: &[u32], weights: &[u16], stride: usize) -> u32 {
        let (mut low, mut high) = (0u32, 0u32);
        for (tap, &weight) in weights.iter().enumerate() {
            let pixel = source[tap * stride];
            let weight = weight as u32;
            low += (pixel & LANES) * weight;
            high += ((pixel >> 8) & LANES) * weight;
        }
        ((low >> 8) & LANES) | (((high >> 8) & LANES) << 8)
    }

    let mut reduced: Vec<u32> = Vec::new();
    let (mut src_w, mut src_h) = (src.width, src.height);
    while src_w >= 2 * width && src_h >= 2 * height {
        let (half_w, half_h) = (src_w.div_ceil(2), src_h.div_ceil(2));
        let current: &[u32] = if reduced.is_empty() { src.pixels } else { &reduced };
        let mut next = vec![0u32; half_w * half_h];
        for y in 0..half_h {
            let top = (y * 2).min(src_h - 1) * src_w;
            let bottom = (y * 2 + 1).min(src_h - 1) * src_w;
            for (x, pixel) in next[y * half_w..][..half_w].iter_mut().enumerate() {
                let left = x * 2;
                let right = (left + 1).min(src_w - 1);
                let (a, b) = (current[top + left], current[top + right]);
                let (c, d) = (current[bottom + left], current[bottom + right]);
                let low = (a & LANES) + (b & LANES) + (c & LANES) + (d & LANES) + 0x0002_0002;
                let high =
                    ((a >> 8) & LANES) + ((b >> 8) & LANES) + ((c >> 8) & LANES) + ((d >> 8) & LANES) + 0x0002_0002;
                *pixel = ((low >> 2) & LANES) | (((high >> 2) & LANES) << 8);
            }
        }
        reduced = next;
        src_w = half_w;
        src_h = half_h;
    }

    let pixels: &[u32] = if reduced.is_empty() { src.pixels } else { &reduced };
    if src_w == width && src_h == height {
        return pixels.to_vec();
    }
    let mut out = vec![0u32; width * height];

    let horizontal = axis(src_w, width);
    let vertical = axis(src_h, height);
    let row_start = vertical.first[0];
    let rows = (vertical.first[height - 1] + vertical.taps).min(src_h) - row_start;

    let mut scratch = vec![0u32; width * rows];
    for row in 0..rows {
        let source_row = &pixels[(row_start + row) * src_w..][..src_w];
        for (i, pixel) in scratch[row * width..][..width].iter_mut().enumerate() {
            let weights = &horizontal.weights[i * horizontal.taps..][..horizontal.taps];
            *pixel = gather(&source_row[horizontal.first[i]..], weights, 1);
        }
    }
    for y in 0..height {
        let weights = &vertical.weights[y * vertical.taps..][..vertical.taps];
        let top = vertical.first[y] - row_start;
        for (x, pixel) in out[y * width..][..width].iter_mut().enumerate() {
            *pixel = gather(&scratch[top * width + x..], weights, width);
        }
    }

    out
}

#[inline(always)]
fn scale_argb(pixel: u32, factor: u32) -> u32 {
    let low = (pixel & LANES) * factor + BIAS;
    let low = (low + ((low >> 8) & LANES)) >> 8 & LANES;
    let high = ((pixel >> 8) & LANES) * factor + BIAS;
    let high = (high + ((high >> 8) & LANES)) >> 8 & LANES;
    low | (high << 8)
}

#[inline(always)]
fn blend(pixel: &mut u32, source: u32, factor: u32) {
    let source = if factor == 255 { source } else { scale_argb(source, factor) };
    *pixel = match source >> 24 {
        255 => source & RGB,
        0 => *pixel,
        alpha => (source + scale_argb(*pixel, 255 - alpha)) & RGB,
    };
}

pub fn draw_image(
    dst: &mut [u32],
    dst_w: usize,
    dst_h: usize,
    src: Image,
    x: i32,
    y: i32,
    w: usize,
    h: usize,
    clip: Rect,
    opacity: u8,
    radius: usize,
    columns: &mut Vec<u32>,
) {
    assert_eq!(dst.len(), dst_w * dst_h);

    let x0 = x.max(clip.x).clamp(0, dst_w as i32) as usize;
    let y0 = y.max(clip.y).clamp(0, dst_h as i32) as usize;
    let x1 = (x as i64 + w as i64).min(clip.right() as i64).clamp(0, dst_w as i64) as usize;
    let y1 = (y as i64 + h as i64).min(clip.bottom() as i64).clamp(0, dst_h as i64) as usize;
    if x0 >= x1 || y0 >= y1 || src.width == 0 || src.height == 0 || opacity == 0 {
        return;
    }

    let step_x = ((src.width << 16) / w) as u32;
    let step_y = ((src.height << 16) / h) as u32;

    columns.clear();
    columns.reserve(x1 - x0);
    let mut acc = (x0 as i64 - x as i64) as u32 * step_x;
    for _ in x0..x1 {
        columns.push((acc >> 16).min(src.width as u32 - 1));
        acc += step_x;
    }

    let radius = radius.min(w / 2).min(h / 2) as f32;
    let centre_x = x as f32 + w as f32 / 2.0;
    let centre_y = y as f32 + h as f32 / 2.0;
    let inset_x = w as f32 / 2.0 - radius;
    let inset_y = h as f32 / 2.0 - radius;
    let opacity = opacity as u32;

    for row in y0..y1 {
        let sy = (((row as i64 - y as i64) as u32 * step_y) >> 16) as usize;
        let src_row = &src.pixels[sy.min(src.height - 1) * src.width..][..src.width];
        let d = row * dst_w + x0;
        let dst_row = &mut dst[d..d + columns.len()];

        if radius == 0.0 {
            for (pixel, &column) in dst_row.iter_mut().zip(&*columns) {
                blend(pixel, src_row[column as usize], opacity);
            }
            continue;
        }

        let distance_y = rounded_axis(row as f32 + 0.5, centre_y, inset_y);
        for (i, (pixel, &column)) in dst_row.iter_mut().zip(&*columns).enumerate() {
            let distance_x = rounded_axis((x0 + i) as f32 + 0.5, centre_x, inset_x);
            let coverage = rounded_coverage(distance_x, distance_y, radius);
            blend(pixel, src_row[column as usize], (opacity as f32 * coverage) as u32);
        }
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (n, entry) in table.iter_mut().enumerate() {
        let mut c = n as u32;
        for _ in 0..8 {
            c = match c & 1 {
                0 => c >> 1,
                _ => 0xEDB8_8320 ^ (c >> 1),
            };
        }
        *entry = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in bytes {
        crc = table[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for chunk in bytes.chunks(5552) {
        for &byte in chunk {
            a += byte as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

pub fn write_png(path: &str, width: usize, height: usize, bgra: &[u8], stride: usize) -> std::io::Result<()> {
    let mut raw = Vec::with_capacity(height * (1 + width * 3));
    for y in 0..height {
        raw.push(0);
        for pixel in bgra[y * stride..y * stride + width * 4].chunks_exact(4) {
            raw.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
        }
    }

    let blocks = raw.len().div_ceil(65535).max(1);
    let mut zlib = vec![0x78, 0x01];
    for (index, block) in raw.chunks(65535).enumerate() {
        zlib.push((index + 1 == blocks) as u8);
        zlib.extend_from_slice(&(block.len() as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);

    let mut out = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &zlib);
    chunk(&mut out, b"IEND", &[]);
    std::fs::write(path, out)
}
