use minwin::Rect;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "png")]
use zune_core::result::DecodingResult;
#[cfg(any(feature = "jpeg", feature = "png"))]
use zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions};
#[cfg(feature = "jpeg")]
use zune_jpeg::{JpegDecoder, errors::DecodeErrors};
#[cfg(feature = "png")]
use zune_png::{PngDecoder, error::PngDecodeErrors};
