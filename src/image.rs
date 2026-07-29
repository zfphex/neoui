use minwin::Rect;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(any(feature = "jpeg", feature = "png"))]
use zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions};
#[cfg(feature = "png")]
use zune_core::result::DecodingResult;
#[cfg(feature = "jpeg")]
use zune_jpeg::{JpegDecoder, errors::DecodeErrors};
#[cfg(feature = "png")]
use zune_png::{PngDecoder, error::PngDecodeErrors};

const LANES: u32 = 0x00FF_00FF;
const BIAS: u32 = 0x0080_0080;
const RGB: u32 = 0x00FF_FFFF;
const WEIGHT_ONE: u32 = 256;
const CACHE_BUDGET: usize = 64 << 20;

static NEXT_IMAGE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct Image {
    pub id: u64,
    pub width: usize,
    pub height: usize,
    pub opaque: bool,
    pub pixels: Box<[u32]>,
}

impl std::hash::Hash for Image {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFit {
    #[default]
    Stretch,
    Contain,
    Cover,
    Fixed,
}

impl Image {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn std::error::Error>> {
        Self::decode(&std::fs::read(path)?)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
            #[cfg(feature = "jpeg")]
            return Ok(decode_jpeg(bytes)?);
            #[cfg(not(feature = "jpeg"))]
            return Err("JPEG decoder is not enabled".into());
        }

        if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            #[cfg(feature = "png")]
            return Ok(decode_png(bytes)?);
            #[cfg(not(feature = "png"))]
            return Err("PNG decoder is not enabled".into());
        }

        Err("unsupported image format".into())
    }

    pub fn from_rgba8(width: usize, height: usize, pixels: &[u8]) -> Self {
        assert_eq!(pixels.len(), width * height * 4);
        Self::packed(
            width,
            height,
            pixels
                .chunks_exact(4)
                .map(|p| premultiply(p[0], p[1], p[2], p[3]))
                .collect(),
        )
    }

    pub fn packed(width: usize, height: usize, pixels: Box<[u32]>) -> Self {
        Self {
            id: NEXT_IMAGE_ID.fetch_add(1, Ordering::Relaxed),
            width,
            height,
            opaque: pixels.iter().all(|p| p >> 24 == 255),
            pixels,
        }
    }

    pub fn thumbnail(&self, size: usize) -> Self {
        let size = size.max(1);
        if self.width == size && self.height == size {
            return self.clone();
        }
        let square = Rect::new(0, 0, size as i32, size as i32);
        let (source, _) = place(self.width, self.height, square, ImageFit::Cover);
        Self::packed(size, size, resample(self, source, size, size).pixels)
    }
}

fn premultiply(r: u8, g: u8, b: u8, a: u8) -> u32 {
    let mul = |v: u8| (v as u32 * a as u32 + 127) / 255;
    (a as u32) << 24 | mul(r) << 16 | mul(g) << 8 | mul(b)
}

#[cfg(feature = "jpeg")]
fn decode_jpeg(bytes: &[u8]) -> Result<Image, DecodeErrors> {
    let settings = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(bytes), settings);
    decoder.decode_headers()?;
    let info = decoder.info().ok_or(DecodeErrors::FormatStatic("missing JPEG dimensions"))?;
    let pixels = decoder.decode()?;
    Ok(Image::from_rgba8(info.width as usize, info.height as usize, &pixels))
}

#[cfg(feature = "png")]
fn decode_png(bytes: &[u8]) -> Result<Image, PngDecodeErrors> {
    let settings = DecoderOptions::default().png_set_strip_to_8bit(true);
    let mut decoder = PngDecoder::new_with_options(ZCursor::new(bytes), settings);
    decoder.decode_headers()?;
    if decoder.is_animated() {
        return Err(PngDecodeErrors::UnsupportedAPNGImage);
    }
    let (width, height) = decoder
        .dimensions()
        .ok_or(PngDecodeErrors::GenericStatic("missing PNG dimensions"))?;
    let color = decoder
        .colorspace()
        .ok_or(PngDecodeErrors::GenericStatic("unknown color space"))?;
    let raw = match decoder.decode()? {
        DecodingResult::U8(data) => data,
        _ => return Err(PngDecodeErrors::GenericStatic("unsupported color space")),
    };

    let pixels: Box<[u32]> = match color {
        ColorSpace::Luma => raw.iter().map(|&v| premultiply(v, v, v, 255)).collect(),
        ColorSpace::LumaA => raw
            .chunks_exact(2)
            .map(|p| premultiply(p[0], p[0], p[0], p[1]))
            .collect(),
        ColorSpace::RGB => raw
            .chunks_exact(3)
            .map(|p| premultiply(p[0], p[1], p[2], 255))
            .collect(),
        ColorSpace::RGBA => raw
            .chunks_exact(4)
            .map(|p| premultiply(p[0], p[1], p[2], p[3]))
            .collect(),
        _ => return Err(PngDecodeErrors::GenericStatic("unsupported color space")),
    };
    assert_eq!(pixels.len(), width * height);
    Ok(Image::packed(width, height, pixels))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Source {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub fn place(width: usize, height: usize, bounds: Rect, fit: ImageFit) -> (Source, Rect) {
    let full = Source {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: height as f32,
    };
    let (iw, ih) = (width as f64, height as f64);
    let (bw, bh) = (bounds.width as f64, bounds.height as f64);

    match fit {
        ImageFit::Stretch => (full, bounds),
        ImageFit::Contain => {
            let scale = (bw / iw).min(bh / ih);
            let dest_width = (iw * scale).round().max(1.0) as i32;
            let dest_height = (ih * scale).round().max(1.0) as i32;
            (
                full,
                Rect::new(
                    bounds.x + (bounds.width - dest_width) / 2,
                    bounds.y + (bounds.height - dest_height) / 2,
                    dest_width,
                    dest_height,
                ),
            )
        }
        ImageFit::Cover => {
            let scale = (bw / iw).max(bh / ih);
            let visible_width = (bw / scale) as f32;
            let visible_height = (bh / scale) as f32;
            (
                Source {
                    x: (width as f32 - visible_width) * 0.5,
                    y: (height as f32 - visible_height) * 0.5,
                    width: visible_width,
                    height: visible_height,
                },
                bounds,
            )
        }
        ImageFit::Fixed => (
            full,
            Rect::new(
                bounds.x + (bounds.width - width as i32) / 2,
                bounds.y + (bounds.height - height as i32) / 2,
                width as i32,
                height as i32,
            ),
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScaleKey {
    pub image: u64,
    pub source: [i32; 4],
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct Scaled {
    pub width: usize,
    pub height: usize,
    pub opaque: bool,
    pub last_used: u32,
    pub pixels: Box<[u32]>,
}

#[derive(Clone, Copy)]
pub struct Pixels<'a> {
    pub data: &'a [u32],
    pub width: usize,
    pub height: usize,
    pub opaque: bool,
}

#[derive(Debug)]
pub struct ImageCache {
    pub entries: FxHashMap<ScaleKey, Scaled>,
    pub frame: u32,
    pub bytes: usize,
    pub budget: usize,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            entries: FxHashMap::default(),
            frame: 0,
            bytes: 0,
            budget: CACHE_BUDGET,
        }
    }

    pub fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        if self.bytes <= self.budget {
            return;
        }
        let mut ages: Vec<(u32, ScaleKey)> = self.entries.iter().map(|(key, e)| (e.last_used, *key)).collect();
        ages.sort_unstable_by_key(|(used, _)| *used);
        for (_, key) in ages {
            if self.bytes <= self.budget {
                break;
            }
            if let Some(entry) = self.entries.remove(&key) {
                self.bytes -= entry.pixels.len() * 4;
            }
        }
    }

    pub fn scaled(&mut self, image: &Image, source: Source, width: usize, height: usize) -> Pixels<'_> {
        let key = ScaleKey {
            image: image.id,
            source: [
                (source.x * 16.0).round() as i32,
                (source.y * 16.0).round() as i32,
                (source.width * 16.0).round() as i32,
                (source.height * 16.0).round() as i32,
            ],
            width: width as u32,
            height: height as u32,
        };
        let frame = self.frame;
        let bytes = &mut self.bytes;
        let entry = self.entries.entry(key).or_insert_with(|| {
            let scaled = resample(image, source, width, height);
            *bytes += scaled.pixels.len() * 4;
            scaled
        });
        entry.last_used = frame;
        Pixels {
            data: &entry.pixels,
            width: entry.width,
            height: entry.height,
            opaque: entry.opaque,
        }
    }
}

pub fn draw_image(
    buffer: &mut [u32],
    framebuffer_width: usize,
    framebuffer_height: usize,
    image: &Image,
    bounds: Rect,
    clip: Rect,
    fit: ImageFit,
    opacity: u8,
    radius: usize,
    scale_factor: f32,
    cache: &mut ImageCache,
) {
    if bounds.is_empty() || clip.is_empty() || opacity == 0 || framebuffer_width == 0 || framebuffer_height == 0 {
        return;
    }

    let (source, placed) = place(image.width, image.height, bounds, fit);
    let dest = placed.scale(scale_factor);
    let draw = dest
        .intersection(bounds.scale(scale_factor))
        .intersection(clip)
        .clamp_to_size(framebuffer_width as i32, framebuffer_height as i32);
    if draw.is_empty() || dest.width <= 0 || dest.height <= 0 {
        return;
    }

    let width = dest.width as usize;
    let height = dest.height as usize;
    let radius = crate::scale(radius, scale_factor).min(width.min(height) / 2);
    let whole_source = source.x == 0.0
        && source.y == 0.0
        && source.width == image.width as f32
        && source.height == image.height as f32;

    if width == image.width && height == image.height && whole_source {
        let pixels = Pixels {
            data: &image.pixels,
            width: image.width,
            height: image.height,
            opaque: image.opaque,
        };
        composite(buffer, framebuffer_width, pixels, dest, draw, opacity, radius);
        return;
    }

    let pixels = cache.scaled(image, source, width, height);
    composite(buffer, framebuffer_width, pixels, dest, draw, opacity, radius);
}

pub struct Axis {
    pub taps: usize,
    pub first: Vec<usize>,
    pub weights: Vec<u16>,
}

fn axis(start: f32, span: f32, source_len: usize, dest_len: usize) -> Axis {
    let ratio = span / dest_len as f32;
    let half = (ratio * 0.5).max(0.5);
    let taps = ((2.0 * half).ceil() as usize + 1).min(source_len).max(1);
    let last_window = source_len - taps;

    let mut first = Vec::with_capacity(dest_len);
    let mut weights = vec![0u16; dest_len * taps];
    let mut scratch = vec![0f32; taps];

    for i in 0..dest_len {
        let center = start + (i as f32 + 0.5) * ratio;
        let (low, high) = (center - half, center + half);
        let begin = low.floor() as i64;
        let window = begin.clamp(0, last_window as i64) as usize;

        scratch.fill(0.0);
        let mut total = 0.0;
        for tap in 0..taps {
            let edge = begin + tap as i64;
            let weight = (((edge + 1) as f32).min(high) - (edge as f32).max(low)).max(0.0);
            let j = edge.clamp(0, source_len as i64 - 1) as usize;
            scratch[(j.saturating_sub(window)).min(taps - 1)] += weight;
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
            let cumulative = exact.round() as u32;
            let weight = cumulative.saturating_sub(assigned).min(WEIGHT_ONE);
            weights[i * taps + tap] = weight as u16;
            assigned += weight;
        }
        first.push(window);
    }

    Axis { taps, first, weights }
}

#[inline(always)]
fn gather(source: &[u32], weights: &[u16], stride: usize) -> u32 {
    if weights.len() == 2 {
        let (a, b) = (source[0], source[stride]);
        let (wa, wb) = (weights[0] as u32, weights[1] as u32);
        let low = (a & LANES) * wa + (b & LANES) * wb;
        let high = ((a >> 8) & LANES) * wa + ((b >> 8) & LANES) * wb;
        return ((low >> 8) & LANES) | (((high >> 8) & LANES) << 8);
    }
    let (mut low, mut high) = (0u32, 0u32);
    for (tap, &weight) in weights.iter().enumerate() {
        let pixel = source[tap * stride];
        let weight = weight as u32;
        low += (pixel & LANES) * weight;
        high += ((pixel >> 8) & LANES) * weight;
    }
    ((low >> 8) & LANES) | (((high >> 8) & LANES) << 8)
}

fn halve(source: &[u32], width: usize, height: usize, dest: &mut [u32], dest_width: usize, dest_height: usize) {
    for y in 0..dest_height {
        let top = (y * 2).min(height - 1) * width;
        let bottom = (y * 2 + 1).min(height - 1) * width;
        let dest_row = &mut dest[y * dest_width..][..dest_width];
        for (x, out) in dest_row.iter_mut().enumerate() {
            let left = x * 2;
            let right = (left + 1).min(width - 1);
            let (a, b) = (source[top + left], source[top + right]);
            let (c, d) = (source[bottom + left], source[bottom + right]);
            let low = (a & LANES) + (b & LANES) + (c & LANES) + (d & LANES) + 0x0002_0002;
            let high = ((a >> 8) & LANES) + ((b >> 8) & LANES) + ((c >> 8) & LANES) + ((d >> 8) & LANES) + 0x0002_0002;
            *out = ((low >> 2) & LANES) | (((high >> 2) & LANES) << 8);
        }
    }
}

fn resample(image: &Image, source: Source, width: usize, height: usize) -> Scaled {
    let mut reduced: Vec<u32> = Vec::new();
    let (mut source_width, mut source_height) = (image.width, image.height);
    let mut rect = source;
    while source_width >= 2
        && source_height >= 2
        && rect.width >= 2.0 * width as f32
        && rect.height >= 2.0 * height as f32
    {
        let (half_width, half_height) = (source_width.div_ceil(2), source_height.div_ceil(2));
        let mut next = vec![0u32; half_width * half_height];
        let current: &[u32] = if reduced.is_empty() { &image.pixels } else { &reduced };
        halve(current, source_width, source_height, &mut next, half_width, half_height);
        reduced = next;
        source_width = half_width;
        source_height = half_height;
        rect = Source {
            x: rect.x * 0.5,
            y: rect.y * 0.5,
            width: rect.width * 0.5,
            height: rect.height * 0.5,
        };
    }

    let exact = rect.x == 0.0 && rect.y == 0.0 && rect.width == width as f32 && rect.height == height as f32;
    if source_width == width && source_height == height && exact && !reduced.is_empty() {
        return Scaled {
            width,
            height,
            opaque: image.opaque,
            last_used: 0,
            pixels: reduced.into_boxed_slice(),
        };
    }
    let pixels_in: &[u32] = if reduced.is_empty() { &image.pixels } else { &reduced };

    let horizontal = axis(rect.x, rect.width, source_width, width);
    let vertical = axis(rect.y, rect.height, source_height, height);

    let row_start = vertical.first[0];
    let row_end = (vertical.first[height - 1] + vertical.taps).min(source_height);
    let rows = row_end - row_start;

    let mut scratch = vec![0u32; width * rows];
    for row in 0..rows {
        let source_row = &pixels_in[(row_start + row) * source_width..][..source_width];
        let dest_row = &mut scratch[row * width..][..width];
        for (i, out) in dest_row.iter_mut().enumerate() {
            let weights = &horizontal.weights[i * horizontal.taps..][..horizontal.taps];
            *out = gather(&source_row[horizontal.first[i]..], weights, 1);
        }
    }

    let mut pixels = vec![0u32; width * height].into_boxed_slice();
    for y in 0..height {
        let weights = &vertical.weights[y * vertical.taps..][..vertical.taps];
        let top = vertical.first[y] - row_start;
        let dest_row = &mut pixels[y * width..][..width];
        for (x, out) in dest_row.iter_mut().enumerate() {
            *out = gather(&scratch[top * width + x..], weights, width);
        }
    }

    Scaled {
        width,
        height,
        opaque: image.opaque,
        last_used: 0,
        pixels,
    }
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
fn over(source: u32, background: u32) -> u32 {
    (source + scale_argb(background, 255 - (source >> 24))) & RGB
}

#[inline]
fn blit_row(dest: &mut [u32], source: &[u32], opacity: u8, opaque: bool) {
    if opacity == 255 {
        if opaque {
            for (d, &s) in dest.iter_mut().zip(source) {
                *d = s & RGB;
            }
        } else {
            for (d, &s) in dest.iter_mut().zip(source) {
                *d = over(s, *d);
            }
        }
        return;
    }
    let opacity = opacity as u32;
    for (d, &s) in dest.iter_mut().zip(source) {
        *d = over(scale_argb(s, opacity), *d);
    }
}

fn composite(
    buffer: &mut [u32],
    framebuffer_width: usize,
    pixels: Pixels<'_>,
    dest: Rect,
    draw: Rect,
    opacity: u8,
    radius: usize,
) {
    let radius = radius.min(pixels.width / 2).min(pixels.height / 2);
    let width = draw.width as usize;

    let top = dest.y + radius as i32;
    let bottom = dest.bottom() - radius as i32;
    let left = (dest.x + radius as i32).clamp(draw.x, draw.right());
    let right = (dest.right() - radius as i32).clamp(left, draw.right());

    let centre_x = dest.x as f32 + pixels.width as f32 / 2.0;
    let centre_y = dest.y as f32 + pixels.height as f32 / 2.0;
    let inset_x = pixels.width as f32 / 2.0 - radius as f32;
    let inset_y = pixels.height as f32 / 2.0 - radius as f32;

    for y in draw.y..draw.bottom() {
        let source_row = (y - dest.y) as usize * pixels.width + (draw.x - dest.x) as usize;
        let dest_row = y as usize * framebuffer_width + draw.x as usize;
        let source = &pixels.data[source_row..][..width];
        let dest_slice = &mut buffer[dest_row..][..width];

        if radius == 0 || (y >= top && y < bottom) {
            blit_row(dest_slice, source, opacity, pixels.opaque);
            continue;
        }

        let lead = (left - draw.x) as usize;
        let trail = (right - draw.x) as usize;
        blit_row(
            &mut dest_slice[lead..trail],
            &source[lead..trail],
            opacity,
            pixels.opaque,
        );

        let distance_y = crate::shapes::rounded_axis(y as f32 + 0.5, centre_y, inset_y);
        for column in (0..lead).chain(trail..width) {
            let x = draw.x + column as i32;
            let distance_x = crate::shapes::rounded_axis(x as f32 + 0.5, centre_x, inset_x);
            let coverage = crate::shapes::rounded_coverage(distance_x, distance_y, radius as f32);
            if coverage <= 0.0 {
                continue;
            }
            let factor = (coverage * opacity as f32).round() as u32;
            dest_slice[column] = over(scale_argb(source[column], factor), dest_slice[column]);
        }
    }
}
