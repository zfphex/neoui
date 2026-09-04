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

#[inline(always)]
fn cubic(x: f32) -> f32 {
    let x = x.abs();
    if x < 1.0 {
        (1.5 * x - 2.5) * x * x + 1.0
    } else if x < 2.0 {
        ((-0.5 * x + 2.5) * x - 4.0) * x + 2.0
    } else {
        0.0
    }
}

struct Axis {
    taps: usize,
    first: Vec<usize>,
    weights: Vec<i16>,
}

fn axis(src_len: usize, dst_len: usize) -> Axis {
    let ratio = src_len as f32 / dst_len as f32;
    let scale = (ratio * 0.5).max(1.0);
    let radius = 2.0 * scale;
    let taps = ((2.0 * radius).ceil() as usize + 1).min(src_len).max(1);
    let inv = 1.0 / scale;
    let max_start = src_len - taps;

    let mut first = Vec::with_capacity(dst_len);
    let mut weights = vec![0i16; dst_len * taps];
    let mut row = vec![0.0f32; taps];

    for i in 0..dst_len {
        let center = (i as f32 + 0.5) * ratio;
        let start = ((center - radius).floor() as i64).clamp(0, max_start as i64) as usize;
        first.push(start);

        let mut sum = 0.0f32;
        for tap in 0..taps {
            let w = cubic(((start + tap) as f32 + 0.5 - center) * inv) * inv;
            row[tap] = w;
            sum += w;
        }

        let norm = if sum != 0.0 { 256.0 / sum } else { 0.0 };
        let mut exact = 0.0f32;
        let mut assigned = 0i32;
        let base = i * taps;
        for tap in 0..taps {
            exact += row[tap] * norm;
            let cum = exact.round() as i32;
            let w = cum - assigned;
            assigned += w;
            weights[base + tap] = w as i16;
        }
    }

    Axis { taps, first, weights }
}

#[inline(always)]
fn gather(source: &[u32], weights: &[i16], stride: usize) -> u32 {
    let mut c0 = 0i32;
    let mut c1 = 0i32;
    let mut c2 = 0i32;
    let mut c3 = 0i32;
    for tap in 0..weights.len() {
        let p = source[tap * stride];
        let w = weights[tap] as i32;
        c0 += (p & 0xFF) as i32 * w;
        c1 += ((p >> 8) & 0xFF) as i32 * w;
        c2 += ((p >> 16) & 0xFF) as i32 * w;
        c3 += ((p >> 24) as i32) * w;
    }
    let b0 = ((c0 + 128) >> 8).clamp(0, 255) as u32;
    let b1 = ((c1 + 128) >> 8).clamp(0, 255) as u32;
    let b2 = ((c2 + 128) >> 8).clamp(0, 255) as u32;
    let b3 = ((c3 + 128) >> 8).clamp(0, 255) as u32;
    b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
}

pub fn resize(src: Image, width: usize, height: usize) -> Vec<u32> {
    assert!(width > 0 && height > 0 && src.width > 0 && src.height > 0);
    if src.width == width && src.height == height {
        return src.pixels.to_vec();
    }

    let horiz = axis(src.width, width);
    let vert = axis(src.height, height);
    let mut scratch = vec![0u32; width * src.height];
    let mut out = vec![0u32; width * height];

    for y in 0..src.height {
        let src_row = &src.pixels[y * src.width..];
        let dst_row = &mut scratch[y * width..];
        for x in 0..width {
            let first = horiz.first[x];
            let weights = &horiz.weights[x * horiz.taps..(x + 1) * horiz.taps];
            dst_row[x] = gather(&src_row[first..], weights, 1);
        }
    }

    for y in 0..height {
        let first = vert.first[y];
        let weights = &vert.weights[y * vert.taps..(y + 1) * vert.taps];
        let dst_row = &mut out[y * width..];
        for x in 0..width {
            dst_row[x] = gather(&scratch[first * width + x..], weights, width);
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
