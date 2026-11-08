//! The Color.js operations used by the Radix generator, in f64 until source storage.
//! Lab mixing uses D50; OKLab/OKLCH use D65. See LICENSES.txt for attribution.
use xgpui_luma::color::ColorValue;
use super::matrices::*;

type Triple = [f64; 3];
const D50: Triple = [0.3457 / 0.3585, 1.0, (1.0 - 0.3457 - 0.3585) / 0.3585];
const KAPPA: f64 = 24389.0 / 27.0;

#[derive(Clone, Copy, Debug)]
pub(super) struct Color {
    pub l: f64,
    pub c: f64,
    /// NaN denotes an achromatic color, as in Color.js.
    pub h: f64,
}

fn multiply(matrix: [[f64; 3]; 3], values: Triple) -> Triple {
    matrix.map(|row| row.iter().zip(values).map(|(a, b)| a * b).sum())
}
fn linearize(v: f64) -> f64 {
    if v.abs() <= 0.04045 {
        v / 12.92
    } else {
        v.signum() * ((v.abs() + 0.055) / 1.055).powf(2.4)
    }
}
fn encode(v: f64) -> f64 {
    if v.abs() <= 0.0031308 {
        v * 12.92
    } else {
        v.signum() * (1.055 * v.abs().powf(1.0 / 2.4) - 0.055)
    }
}
impl Color {
    pub fn from_source(source: ColorValue) -> anyhow::Result<Self> {
        source.validate()?;
        let color = match source {
            ColorValue::Srgb(c) => Self::from_rgb([c.red as f64, c.green as f64, c.blue as f64]),
            ColorValue::LinearSrgb(c) => {
                Self::from_xyz(multiply(RGB_TO_XYZ, [c.red as f64, c.green as f64, c.blue as f64]))
            }
            ColorValue::DisplayP3(c) => Self::from_p3([c.red as f64, c.green as f64, c.blue as f64]),
            ColorValue::Oklch(c) => Self { l: c.l as f64, c: c.chroma as f64, h: c.hue.into_raw_degrees() as f64 },
        };
        color.source().validate()?;
        Ok(color)
    }

    /// Derived colors use Oklch; internal missing achromatic hue resolves to zero.
    pub fn source(self) -> ColorValue {
        ColorValue::oklch(self.l as f32, self.c as f32, if self.h.is_nan() { 0.0 } else { self.h as f32 }, 1.0)
    }
    pub fn from_rgb(rgb: Triple) -> Self {
        Self::from_xyz(multiply(RGB_TO_XYZ, rgb.map(linearize)))
    }
    pub fn from_p3(rgb: Triple) -> Self {
        Self::from_xyz(multiply(P3_TO_XYZ, rgb.map(linearize)))
    }
    fn from_xyz(xyz: Triple) -> Self {
        let [l, a, b] = multiply(LMS_TO_OKLAB, multiply(XYZ_TO_LMS, xyz).map(f64::cbrt));
        Self {
            l,
            c: a.hypot(b),
            h: if a.abs() < 0.0002 && b.abs() < 0.0002 {
                f64::NAN
            } else {
                b.atan2(a).to_degrees().rem_euclid(360.0)
            },
        }
    }
    fn oklab(self) -> Triple {
        // Color.js resolves missing coordinates to zero before space conversion.
        let hue = if self.h.is_nan() { 0.0 } else { self.h.to_radians() };
        let (a, b) = (self.c * hue.cos(), self.c * hue.sin());
        [self.l, a, b]
    }
    fn xyz(self) -> Triple {
        multiply(LMS_TO_XYZ, multiply(OKLAB_TO_LMS, self.oklab()).map(|v| v.powi(3)))
    }
    pub fn rgb(self) -> Triple {
        multiply(XYZ_TO_RGB, self.xyz()).map(encode)
    }
    pub fn distance(self, other: Self) -> f64 {
        self.oklab().iter().zip(other.oklab()).map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt()
    }
    fn lab(self) -> Triple {
        let xyz = multiply(D65_TO_D50, self.xyz());
        let f: Triple = std::array::from_fn(|i| {
            let v = xyz[i] / D50[i];
            if v > 216.0 / 24389.0 {
                v.cbrt()
            } else {
                (KAPPA * v + 16.0) / 116.0
            }
        });
        [116.0 * f[1] - 16.0, 500.0 * (f[0] - f[1]), 200.0 * (f[1] - f[2])]
    }
    /// Color.mix in Color.js defaults to CIE Lab, not OKLCH.
    pub fn mix(self, other: Self, ratio: f64) -> Self {
        let a = self.lab();
        let b = other.lab();
        let [l, a, b] = std::array::from_fn(|i| a[i] + (b[i] - a[i]) * ratio);
        let fy = (l + 16.0) / 116.0;
        let f = [a / 500.0 + fy, fy, fy - b / 200.0];
        let xyz = std::array::from_fn(|i| {
            let v = if i == 1 {
                if l > 8.0 { fy.powi(3) } else { l / KAPPA }
            } else if f[i] > 24.0 / 116.0 {
                f[i].powi(3)
            } else {
                (116.0 * f[i] - 16.0) / KAPPA
            };
            v * D50[i]
        });
        Self::from_xyz(multiply(D50_TO_D65, xyz))
    }
    pub fn text_color(self) -> Self {
        // Preserve upstream's white.contrastAPCA(source) call order (white background).
        let rgb = self.rgb();
        let y = rgb
            .iter()
            .zip([0.2126729, 0.7151522, 0.0721750])
            .map(|(v, w)| v.signum() * v.abs().powf(2.4) * w)
            .sum::<f64>();
        let y = if y >= 0.022 { y } else { y + (0.022 - y).powf(1.414) };
        let white_y: f64 = 0.2126729 + 0.7151522 + 0.0721750;
        let contrast = if (white_y - y).abs() < 0.0005 {
            0.0
        } else if white_y > y {
            (white_y.powf(0.56) - y.powf(0.57)) * 1.14
        } else {
            (white_y.powf(0.65) - y.powf(0.62)) * 1.14
        };
        let contrast = if contrast.abs() < 0.1 {
            0.0
        } else {
            (contrast - contrast.signum() * 0.027) * 100.0
        };
        if contrast.abs() < 40.0 {
            Self { l: 0.25, c: (0.08 * self.c).max(0.04), h: self.h }
        } else {
            Self { l: 1.0, c: 0.0, h: f64::NAN }
        }
    }
}
