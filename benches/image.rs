use divan::black_box;
use neoui::*;
use rustc_hash::FxHashMap;

fn main() {
    divan::main();
}

const FB_W: usize = 1920;
const FB_H: usize = 1080;

fn full() -> Rect {
    Rect::new(0, 0, FB_W as i32, FB_H as i32)
}

fn image(width: usize, height: usize, alpha: u8) -> Image {
    let mut pixels = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&[(x * 7) as u8, (y * 5) as u8, (x ^ y) as u8, alpha]);
        }
    }
    Image::from_rgba8(width, height, &pixels)
}

struct Bed {
    buffer: Vec<u32>,
    cache: ImageCache,
    fonts: Vec<fontdue::Font>,
    glyphs: FxHashMap<(usize, char, usize), (fontdue::Metrics, Vec<u8>)>,
}

impl Bed {
    fn new() -> Self {
        Self {
            buffer: vec![0; FB_W * FB_H],
            cache: ImageCache::new(),
            fonts: Vec::new(),
            glyphs: FxHashMap::default(),
        }
    }

    fn draw(&mut self, image: &Image, bounds: Rect, fit: ImageFit, opacity: u8, radius: usize) {
        let command = Command::Image {
            image,
            bounds,
            clip: full(),
            fit,
            opacity,
            radius,
        };
        draw_command(
            black_box(&command),
            full(),
            &mut self.buffer,
            FB_W,
            FB_H,
            1.0,
            &self.fonts,
            &[],
            &mut self.glyphs,
            &mut self.cache,
        );
    }
}

mod warm {
    use super::*;

    fn bench(bencher: divan::Bencher, src: (usize, usize), dst: i32, fit: ImageFit, opacity: u8, radius: usize) {
        let img = image(src.0, src.1, 255);
        let bounds = Rect::new(0, 0, dst, dst);
        bencher
            .with_inputs(|| {
                let mut bed = Bed::new();
                bed.draw(&img, bounds, fit, opacity, radius);
                bed
            })
            .bench_local_refs(|bed| bed.draw(&img, bounds, fit, opacity, radius));
    }

    #[divan::bench]
    fn blit_1to1(bencher: divan::Bencher) {
        bench(bencher, (512, 512), 512, ImageFit::Stretch, 255, 0);
    }

    #[divan::bench]
    fn blit_1to1_alpha(bencher: divan::Bencher) {
        let img = image(512, 512, 128);
        let bounds = Rect::new(0, 0, 512, 512);
        bencher
            .with_inputs(|| {
                let mut bed = Bed::new();
                bed.draw(&img, bounds, ImageFit::Stretch, 255, 0);
                bed
            })
            .bench_local_refs(|bed| bed.draw(&img, bounds, ImageFit::Stretch, 255, 0));
    }

    #[divan::bench]
    fn blit_1to1_opacity(bencher: divan::Bencher) {
        bench(bencher, (512, 512), 512, ImageFit::Stretch, 128, 0);
    }

    #[divan::bench]
    fn blit_1to1_radius(bencher: divan::Bencher) {
        bench(bencher, (512, 512), 512, ImageFit::Stretch, 255, 16);
    }

    #[divan::bench]
    fn downscale_4x(bencher: divan::Bencher) {
        bench(bencher, (2048, 2048), 512, ImageFit::Stretch, 255, 0);
    }

    #[divan::bench]
    fn upscale_4x(bencher: divan::Bencher) {
        bench(bencher, (128, 128), 512, ImageFit::Stretch, 255, 0);
    }
}

mod cold {
    use super::*;

    fn bench(bencher: divan::Bencher, src: (usize, usize), dst: i32, fit: ImageFit, radius: usize) {
        let img = image(src.0, src.1, 255);
        let bounds = Rect::new(0, 0, dst, dst);
        bencher.with_inputs(Bed::new).bench_local_refs(|bed| {
            bed.cache.entries.clear();
            bed.cache.bytes = 0;
            bed.draw(&img, bounds, fit, 255, radius);
        });
    }

    #[divan::bench]
    fn blit_1to1(bencher: divan::Bencher) {
        bench(bencher, (512, 512), 512, ImageFit::Stretch, 0);
    }

    #[divan::bench]
    fn downscale_4x(bencher: divan::Bencher) {
        bench(bencher, (2048, 2048), 512, ImageFit::Stretch, 0);
    }

    #[divan::bench]
    fn downscale_4x_radius(bencher: divan::Bencher) {
        bench(bencher, (2048, 2048), 512, ImageFit::Stretch, 16);
    }

    #[divan::bench]
    fn upscale_4x(bencher: divan::Bencher) {
        bench(bencher, (128, 128), 512, ImageFit::Stretch, 0);
    }

    #[divan::bench]
    fn cover_crop(bencher: divan::Bencher) {
        bench(bencher, (2048, 1024), 512, ImageFit::Cover, 0);
    }
}

mod animated {
    use super::*;

    #[divan::bench]
    fn fade(bencher: divan::Bencher) {
        let img = image(512, 512, 255);
        let bounds = Rect::new(0, 0, 512, 512);
        let mut frame = 0u32;
        bencher.with_inputs(Bed::new).bench_local_refs(|bed| {
            frame = frame.wrapping_add(1);
            let opacity = (frame % 255) as u8 + 1;
            bed.draw(&img, bounds, ImageFit::Stretch, opacity, 0);
        });
    }

    #[divan::bench]
    fn resize(bencher: divan::Bencher) {
        let img = image(1024, 1024, 255);
        let mut frame = 0u32;
        bencher.with_inputs(Bed::new).bench_local_refs(|bed| {
            frame = frame.wrapping_add(1);
            let side = 256 + (frame % 256) as i32;
            bed.draw(&img, Rect::new(0, 0, side, side), ImageFit::Stretch, 255, 0);
        });
    }
}
