use crate::Rect;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_IMAGE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct Image {
    pub id: u64,
    pub width: usize,
    pub height: usize,
    ///ARGB
    pub pixels: Box<[u32]>,
}

impl std::hash::Hash for Image {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFit {
    #[default]
    Stretch,
    Contain,
    Cover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageStyle {
    pub fit: ImageFit,
    pub opacity: u8,
    pub radius: usize,
    pub depth: usize,
}

impl Default for ImageStyle {
    fn default() -> Self {
        Self {
            fit: ImageFit::Stretch,
            opacity: 255,
            radius: 0,
            depth: 0,
        }
    }
}

impl Image {
    pub fn from_rgba8(width: usize, height: usize, pixels: impl Into<Box<[u8]>>) -> Result<Self, String> {
        let pixels = pixels.into();
        let count = checked_pixels(width, height)?;
        let expected = count
            .checked_mul(4)
            .ok_or_else(|| "image dimensions overflow addressable memory".to_string())?;
        if pixels.len() != expected {
            return Err(format!(
                "invalid image buffer length: expected {expected}, got {}",
                pixels.len()
            ));
        }
        let packed = pixels.chunks(4).map(|p| premultiply(p[0], p[1], p[2], p[3])).collect();
        Ok(Self::new(width, height, packed))
    }

    pub fn from_argb32(width: usize, height: usize, pixels: impl Into<Box<[u32]>>) -> Result<Self, String> {
        let pixels = pixels.into();
        let expected = checked_pixels(width, height)?;
        if pixels.len() != expected {
            return Err(format!(
                "invalid image buffer length: expected {expected}, got {}",
                pixels.len()
            ));
        }
        let packed = pixels
            .iter()
            .map(|p| premultiply((p >> 16) as u8, (p >> 8) as u8, *p as u8, (p >> 24) as u8))
            .collect();
        Ok(Self::new(width, height, packed))
    }

    fn new(width: usize, height: usize, pixels: Box<[u32]>) -> Self {
        Self {
            id: NEXT_IMAGE_ID
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .unwrap(),
            width,
            height,
            pixels,
        }
    }

    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        Self::decode(&std::fs::read(path).map_err(|error| error.to_string())?)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
            #[cfg(feature = "jpeg")]
            return decode_jpeg(bytes);
            #[cfg(not(feature = "jpeg"))]
            return Err(format!("JPEG decoder is not enabled"));
        }

        if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            #[cfg(feature = "png")]
            return decode_png(bytes);
            #[cfg(not(feature = "png"))]
            return Err(format!("PNG decoder is not enabled"));
        }

        Err("unsupported image format".to_string())
    }
}

fn checked_pixels(width: usize, height: usize) -> Result<usize, String> {
    if width == 0 || height == 0 {
        return Err("image dimensions must be non-zero".into());
    }
    width
        .checked_mul(height)
        .ok_or_else(|| "image dimensions overflow addressable memory".to_string())
}

#[cfg(any(feature = "jpeg", feature = "png"))]
fn validate_decode_size(width: usize, height: usize) -> Result<(), String> {
    let count = checked_pixels(width, height)?;
    count
        .checked_mul(4)
        .ok_or_else(|| "image dimensions overflow addressable memory".to_string())?;
    Ok(())
}

#[cfg(feature = "jpeg")]
fn decode_jpeg(bytes: &[u8]) -> Result<Image, String> {
    use zune_jpeg::zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions};
    let settings = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGBA);
    let mut decoder = zune_jpeg::JpegDecoder::new_with_options(ZCursor::new(bytes), settings);
    decoder
        .decode_headers()
        .map_err(|e| decode_error(ImageFormat::Jpeg, e))?;
    let info = decoder
        .info()
        .ok_or_else(|| "failed to decode Jpeg: missing JPEG dimensions".to_string())?;
    let (width, height) = (info.width as usize, info.height as usize);
    validate_decode_size(width, height)?;
    let pixels = decoder.decode().map_err(|e| decode_error(ImageFormat::Jpeg, e))?;
    Image::from_rgba8(width, height, pixels)
}

#[cfg(feature = "png")]
fn decode_png(bytes: &[u8]) -> Result<Image, String> {
    use zune_png::zune_core::{
        bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions, result::DecodingResult,
    };
    let settings = DecoderOptions::default().png_set_strip_to_8bit(true);
    let mut decoder = zune_png::PngDecoder::new_with_options(ZCursor::new(bytes), settings);
    decoder
        .decode_headers()
        .map_err(|e| decode_error(ImageFormat::Png, e))?;
    if decoder.is_animated() {
        return Err("animated PNG images are not supported".into());
    }
    let (width, height) = decoder
        .dimensions()
        .ok_or_else(|| "failed to decode Png: missing PNG dimensions".to_string())?;
    validate_decode_size(width, height)?;
    let color = decoder
        .colorspace()
        .ok_or_else(|| "unsupported decoded image color space".to_string())?;
    let raw = match decoder.decode().map_err(|e| decode_error(ImageFormat::Png, e))? {
        DecodingResult::U8(data) => data,
        _ => return Err("unsupported decoded image color space".into()),
    };
    let mut rgba = Vec::with_capacity(width * height * 4);
    match color {
        ColorSpace::Luma => {
            for &v in &raw {
                rgba.extend_from_slice(&[v, v, v, 255]);
            }
        }
        ColorSpace::LumaA => {
            for p in raw.chunks(2) {
                rgba.extend_from_slice(&[p[0], p[0], p[0], p[1]]);
            }
        }
        ColorSpace::RGB => {
            for p in raw.chunks(3) {
                rgba.extend_from_slice(&[p[0], p[1], p[2], 255]);
            }
        }
        ColorSpace::RGBA => rgba = raw,
        _ => return Err("unsupported decoded image color space".into()),
    }
    Image::from_rgba8(width, height, rgba)
}

#[cfg(any(feature = "jpeg", feature = "png"))]
fn decode_error(format: ImageFormat, error: impl std::fmt::Debug) -> String {
    format!("failed to decode {format:?}: {error:?}")
}

pub fn fitted_bounds(bounds: Rect, image: &Image, fit: ImageFit) -> Rect {
    if bounds.is_empty() || fit != ImageFit::Contain {
        return bounds;
    }
    let scale = (bounds.width as f64 / image.width as f64).min(bounds.height as f64 / image.height as f64);
    let width = (image.width as f64 * scale).round().max(1.0) as i32;
    let height = (image.height as f64 * scale).round().max(1.0) as i32;
    Rect::new(
        bounds.x + (bounds.width - width) / 2,
        bounds.y + (bounds.height - height) / 2,
        width,
        height,
    )
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
) {
    if bounds.is_empty() || clip.is_empty() || opacity == 0 || framebuffer_width == 0 || framebuffer_height == 0 {
        return;
    }
    let target = fitted_bounds(bounds, image, fit);
    let draw = target
        .intersection(clip)
        .clamp_to_size(framebuffer_width as i32, framebuffer_height as i32);
    if draw.is_empty() {
        return;
    }
    let radius = radius.min((target.width.min(target.height).max(0) / 2) as usize) as f32;
    let (scale_x, scale_y) = match fit {
        ImageFit::Stretch | ImageFit::Contain => (
            image.width as f32 / target.width as f32,
            image.height as f32 / target.height as f32,
        ),
        ImageFit::Cover => {
            let scale = (bounds.width as f32 / image.width as f32).max(bounds.height as f32 / image.height as f32);
            (1.0 / scale, 1.0 / scale)
        }
    };
    let visible_src_w = target.width as f32 * scale_x;
    let visible_src_h = target.height as f32 * scale_y;
    let src_x0 = (image.width as f32 - visible_src_w) * 0.5;
    let src_y0 = (image.height as f32 - visible_src_h) * 0.5;

    for y in draw.y..draw.bottom() {
        for x in draw.x..draw.right() {
            let coverage = rounded_coverage(x, y, target, radius);
            if coverage <= 0.0 {
                continue;
            }
            let sx = src_x0 + (x as f32 + 0.5 - target.x as f32) * scale_x - 0.5;
            let sy = src_y0 + (y as f32 + 0.5 - target.y as f32) * scale_y - 0.5;
            let pixel = sample_bilinear(image, sx, sy);
            let alpha_scale = opacity as f32 / 255.0 * coverage;
            let a = (((pixel >> 24) & 255) as f32 * alpha_scale).round() as u32;
            if a == 0 {
                continue;
            }
            let r = (((pixel >> 16) & 255) as f32 * alpha_scale).round() as u32;
            let g = (((pixel >> 8) & 255) as f32 * alpha_scale).round() as u32;
            let b = ((pixel & 255) as f32 * alpha_scale).round() as u32;
            let index = y as usize * framebuffer_width + x as usize;
            if a >= 255 {
                buffer[index] = r << 16 | g << 8 | b;
                continue;
            }
            let bg = buffer[index];
            let inv = 255 - a;
            let br = (bg >> 16) & 255;
            let bgc = (bg >> 8) & 255;
            let bb = bg & 255;
            buffer[index] =
                (r + (br * inv + 127) / 255) << 16 | (g + (bgc * inv + 127) / 255) << 8 | (b + (bb * inv + 127) / 255);
        }
    }
}

fn premultiply(r: u8, g: u8, b: u8, a: u8) -> u32 {
    let mul = |v: u8| (v as u32 * a as u32 + 127) / 255;
    (a as u32) << 24 | mul(r) << 16 | mul(g) << 8 | mul(b)
}

fn sample_bilinear(image: &Image, x: f32, y: f32) -> u32 {
    let x = x.clamp(0.0, (image.width - 1) as f32);
    let y = y.clamp(0.0, (image.height - 1) as f32);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(image.width - 1);
    let y1 = (y0 + 1).min(image.height - 1);
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let pixels = [
        image.pixels[y0 * image.width + x0],
        image.pixels[y0 * image.width + x1],
        image.pixels[y1 * image.width + x0],
        image.pixels[y1 * image.width + x1],
    ];
    let mut out = 0;
    for shift in [0, 8, 16, 24] {
        let a = ((pixels[0] >> shift) & 255) as f32 * (1.0 - fx) + ((pixels[1] >> shift) & 255) as f32 * fx;
        let b = ((pixels[2] >> shift) & 255) as f32 * (1.0 - fx) + ((pixels[3] >> shift) & 255) as f32 * fx;
        out |= (a.mul_add(1.0 - fy, b * fy).round() as u32) << shift;
    }
    out
}

fn rounded_coverage(x: i32, y: i32, bounds: Rect, radius: f32) -> f32 {
    if radius <= 0.0 {
        return 1.0;
    }
    let px = x as f32 + 0.5;
    let py = y as f32 + 0.5;
    let cx = px.clamp(bounds.x as f32 + radius, bounds.right() as f32 - radius);
    let cy = py.clamp(bounds.y as f32 + radius, bounds.bottom() as f32 - radius);
    let distance = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
    (radius + 0.5 - distance).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_validate_and_preserve_identity_on_clone() {
        assert_eq!(
            Image::from_rgba8(0, 1, []).unwrap_err().to_string(),
            "image dimensions must be non-zero"
        );
        assert_eq!(
            Image::from_rgba8(1, 1, [0, 0, 0]).unwrap_err().to_string(),
            "invalid image buffer length: expected 4, got 3"
        );
        let image = Image::from_rgba8(1, 1, [200, 100, 50, 128]).unwrap();
        assert_eq!(image.id, image.clone().id);
        assert_eq!(image.pixels[0], 0x8064_3219);
        assert_ne!(image.id, Image::from_argb32(1, 1, [0xffff_ffff]).unwrap().id);
    }

    #[test]
    fn blending_and_contain_are_correct() {
        let image = Image::from_rgba8(1, 1, [255, 0, 0, 128]).unwrap();
        let mut buffer = vec![0x0000ff; 4];
        draw_image(
            &mut buffer,
            2,
            2,
            &image,
            Rect::new(0, 0, 2, 2),
            Rect::new(0, 0, 2, 2),
            ImageFit::Stretch,
            255,
            0,
        );
        assert_eq!(buffer, vec![0x80007f; 4]);
        let wide = Image::from_rgba8(2, 1, [255, 255, 255, 255, 255, 255, 255, 255]).unwrap();
        assert_eq!(
            fitted_bounds(Rect::new(0, 0, 4, 4), &wide, ImageFit::Contain),
            Rect::new(0, 1, 4, 2)
        );
    }

    #[cfg(feature = "png")]
    #[test]
    fn decodes_png() {
        use zune_png::zune_core::{bit_depth::BitDepth, colorspace::ColorSpace, options::EncoderOptions};
        let options = EncoderOptions::default()
            .set_colorspace(ColorSpace::RGBA)
            .set_width(1)
            .set_height(1)
            .set_depth(BitDepth::Eight);
        let mut encoded = Vec::new();
        zune_png::PngEncoder::new(&[10, 20, 30, 128], options)
            .encode(&mut encoded)
            .unwrap();
        let image = Image::decode(&encoded).unwrap();
        assert_eq!((image.width, image.height), (1, 1));
    }

    #[cfg(all(feature = "jpeg", not(feature = "png")))]
    #[test]
    fn recognized_disabled_png_is_reported() {
        assert_eq!(
            Image::decode(b"\x89PNG\r\n\x1a\n").unwrap_err().to_string(),
            "Png decoder is not enabled"
        );
    }

    #[cfg(all(feature = "png", not(feature = "jpeg")))]
    #[test]
    fn recognized_disabled_jpeg_is_reported() {
        assert_eq!(
            Image::decode(&[0xff, 0xd8, 0xff]).unwrap_err().to_string(),
            "Jpeg decoder is not enabled"
        );
    }

    #[test]
    fn unknown_signature_is_rejected() {
        assert_eq!(
            Image::decode(b"not an image").unwrap_err().to_string(),
            "unsupported image format"
        );
    }
}
