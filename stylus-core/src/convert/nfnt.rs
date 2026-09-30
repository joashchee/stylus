//! nfnt/resize by Jan Schlicht (github.com/nfnt/resize, ISC), which
//! several of the Go converters resize with, ported from `resize.go`,
//! `filters.go`, `converter.go`, `nearest.go` and `thumbnail.go`, with
//! Go's color model around it: images are `*image.NRGBA` (straight alpha,
//! as Go's PNG decoder returns) or `*image.RGBA` (premultiplied, as the
//! resize returns), and `rgba()` is Go's `Color.RGBA()`.
//!
//! Go's JPEG decoder yields YCbCr, which nfnt resizes as YCbCr; here every
//! image is resized as RGB(A), so JPEG results can differ slightly.

use image::RgbaImage;

/// A decoded or resized Go image.
#[derive(Clone)]
pub struct GoImage {
    pub w: u32,
    pub h: u32,
    pub px: Vec<[u8; 4]>,
    /// `*image.RGBA` (premultiplied) rather than `*image.NRGBA`.
    pub premultiplied: bool,
}

impl GoImage {
    /// An image as Go's decoders return it: straight alpha. (An opaque
    /// image resizes the same either way.)
    pub fn from_straight(img: &RgbaImage) -> GoImage {
        GoImage {
            w: img.width(),
            h: img.height(),
            px: img.pixels().map(|p| p.0).collect(),
            premultiplied: false,
        }
    }

    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.px[(y * self.w + x) as usize]
    }

    /// `img.At(x, y).RGBA()`: premultiplied, 16 bits per channel.
    pub fn rgba(&self, x: u32, y: u32) -> [u32; 4] {
        let p = self.pixel(x, y);
        if self.premultiplied {
            return p.map(|v| v as u32 * 0x101);
        }
        let a = p[3] as u32 * 0x101;
        [p[0] as u32 * 0x101 * a / 0xffff, p[1] as u32 * 0x101 * a / 0xffff, p[2] as u32 * 0x101 * a / 0xffff, a]
    }

}

#[derive(Clone, Copy, PartialEq)]
pub enum Interp {
    NearestNeighbor,
    Lanczos3,
}

impl Interp {
    fn taps(self) -> usize {
        match self {
            Interp::NearestNeighbor => 2,
            Interp::Lanczos3 => 6,
        }
    }

    fn kernel(self, x: f64) -> f64 {
        match self {
            Interp::NearestNeighbor => {
                if (-0.5..0.5).contains(&x) {
                    1.0
                } else {
                    0.0
                }
            }
            Interp::Lanczos3 => {
                if x > -3.0 && x < 3.0 {
                    sinc(x) * sinc(x * 0.3333333333333333)
                } else {
                    0.0
                }
            }
        }
    }
}

fn sinc(x: f64) -> f64 {
    let x = x.abs() * std::f64::consts::PI;
    if x >= 1.220703e-4 {
        super::util::go_sin(x) / x
    } else {
        1.0
    }
}

/// `createWeights8` (or `createWeightsNearest`): per output pixel, the
/// first source index and the weights, as i16 (×256, truncated).
fn weights(dy: usize, interp: Interp, scale: f64) -> (Vec<i16>, Vec<i64>, usize) {
    let blur = 1.0;
    let filter_length = interp.taps() * ((blur * scale).ceil().max(1.0) as usize);
    let filter_factor = (1.0 / (blur * scale)).min(1.0);
    let mut coeffs = vec![0i16; dy * filter_length];
    let mut start = vec![0i64; dy];
    for y in 0..dy {
        let mut interp_x = scale * (y as f64 + 0.5) - 0.5;
        start[y] = interp_x as i64 - (filter_length / 2) as i64 + 1;
        interp_x -= start[y] as f64;
        for i in 0..filter_length {
            let k = interp.kernel((interp_x - i as f64) * filter_factor);
            coeffs[y * filter_length + i] = if interp == Interp::NearestNeighbor { k as i16 } else { (k * 256.0) as i16 };
        }
    }
    (coeffs, start, filter_length)
}

/// One pass: filters each source row into a column of the (transposed)
/// output, `out_len` values per row. `premultiply` does NRGBA's forward
/// premultiplication; `nearest` averages the taps in f32 as nfnt's
/// nearest-neighbor does.
fn pass(src: &[[u8; 4]], sw: usize, sh: usize, out_len: usize, interp: Interp, scale: f64, premultiply: bool) -> Vec<[u8; 4]> {
    let (coeffs, start, fl) = weights(out_len, interp, scale);
    let max_x = sw as i64 - 1;
    // Output is transposed: `sh` wide, `out_len` tall.
    let mut out = vec![[0u8; 4]; sh * out_len];
    for x in 0..sh {
        let row = &src[x * sw..(x + 1) * sw];
        for y in 0..out_len {
            let mut acc = [0i32; 4];
            let mut accf = [0f32; 4];
            let mut sum = 0i32;
            for i in 0..fl {
                let c = coeffs[y * fl + i];
                if c == 0 {
                    continue;
                }
                let xi = start[y] + i as i64;
                // `uint(xi) < uint(maxX)`, then clamp high, else 0.
                let xi = if xi >= 0 && xi < max_x {
                    xi
                } else if xi >= max_x {
                    max_x
                } else {
                    0
                } as usize;
                let p = row[xi];
                if interp == Interp::NearestNeighbor {
                    for ch in 0..4 {
                        accf[ch] += p[ch] as f32;
                    }
                    sum += 1;
                } else {
                    let v = if premultiply {
                        let a = p[3] as i32;
                        [p[0] as i32 * a / 0xff, p[1] as i32 * a / 0xff, p[2] as i32 * a / 0xff, a]
                    } else {
                        p.map(|v| v as i32)
                    };
                    for ch in 0..4 {
                        acc[ch] += c as i32 * v[ch];
                    }
                    sum += c as i32;
                }
            }
            out[y * sh + x] = if interp == Interp::NearestNeighbor {
                // floatToUint8
                accf.map(|v| {
                    let v = v / sum as f32;
                    if v > 254.0 {
                        255
                    } else {
                        v as u8
                    }
                })
            } else {
                // clampUint8 of a truncating division.
                acc.map(|v| (v / sum).clamp(0, 255) as u8)
            };
        }
    }
    out
}

/// `resize.Resize(width, height, img, interp)`.
pub fn resize(width: u32, height: u32, img: &GoImage, interp: Interp) -> GoImage {
    if (width, height) == (img.w, img.h) {
        return img.clone();
    }
    let scale_x = img.w as f64 / width as f64;
    let scale_y = img.h as f64 / height as f64;
    let nearest = interp == Interp::NearestNeighbor;
    // Horizontal, into a transposed temporary; then the same on it.
    let premultiply = !img.premultiplied && !nearest;
    let temp = pass(&img.px, img.w as usize, img.h as usize, width as usize, interp, scale_x, premultiply);
    let out = pass(&temp, img.h as usize, width as usize, height as usize, interp, scale_y, false);
    GoImage {
        w: width,
        h: height,
        px: out,
        // NRGBA stays NRGBA only through the nearest-neighbor path.
        premultiplied: img.premultiplied || !nearest,
    }
}

/// `resize.Thumbnail`: shrinks to fit, keeping the aspect ratio; never
/// enlarges.
pub fn thumbnail(max_width: u32, max_height: u32, img: &GoImage, interp: Interp) -> GoImage {
    let (ow, oh) = (img.w as u64, img.h as u64);
    if max_width as u64 >= ow && max_height as u64 >= oh {
        return img.clone();
    }
    let (mut nw, mut nh) = (ow, oh);
    if ow > max_width as u64 {
        nh = (oh * max_width as u64 / ow).max(1);
        nw = max_width as u64;
    }
    if nh > max_height as u64 {
        nw = (nw * max_height as u64 / nh).max(1);
        nh = max_height as u64;
    }
    resize(nw as u32, nh as u32, img, interp)
}
