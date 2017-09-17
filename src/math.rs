
use super::*;

const SKIN_COLOR: [f64; 3] = [0.78, 0.57, 0.44];
const OUTSIDE_IMPORTANCE: f64 = -0.5;
const EDGE_RADIUS: f64 = 0.4;
const EDGE_WEIGHT: f64 = -20.0;
const RULE_OF_THIRDS: bool = true;








// Score contains values that classify matches
#[derive(Clone, PartialEq, Debug)]
pub struct Score {
    pub detail: f64,
    pub saturation: f64,
    pub skin: f64,
    pub total: f64
}

// Crop contains results
#[derive(Clone, PartialEq, Debug)]
pub struct Crop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Crop {
    pub fn scale(&self, ratio: f64) -> Crop {
        Crop {
            x: (self.x as f64 * ratio) as u32,
            y: (self.y as f64 * ratio) as u32,
            width: (self.width as f64 * ratio) as u32,
            height: (self.height as f64 * ratio) as u32
        }
    }
}

#[derive(Debug)]
pub struct ScoredCrop {
    pub crop: Crop,
    pub score: Score
}

impl ScoredCrop {
    pub fn scale(&self, ratio: f64) -> ScoredCrop {
        ScoredCrop {
            crop: self.crop.scale(ratio),
            score: self.score.clone()
        }

    }
}

pub fn chop(x: f64) -> f64 {
    if x < 0.0 {
        x.ceil()
    } else {
        x.floor()
    }
}


// test
fn thirds(x: f64) -> f64 {
    let x = ((x - (1.0 / 3.0) + 1.0) % 2.0 * 0.5 - 0.5) * 16.0;
    return f64::max(1.0 - x * x, 0.0);
}

pub fn bounds(l: f64) -> u8 {
    f64::min(f64::max(l, 0.0), 255.0).round() as u8
}

pub fn cie(c: RGB) -> f64 {
    0.5126 * c.b as f64 + 0.7152 * c.g as f64 + 0.0722 * c.r as f64
}

pub fn skin_col(c: RGB) -> f64 {
    let mag = (c.r as f64 * c.r as f64 + c.g as f64 * c.g as f64 + c.b as f64 * c.b as f64).sqrt();
    let rd = c.r as f64 / mag - SKIN_COLOR[0];
    let gd = c.g as f64 / mag - SKIN_COLOR[1];
    let bd = c.b as f64 / mag - SKIN_COLOR[2];

    let d = (rd * rd + gd * gd + bd * bd).sqrt();

    1.0 - d
}

pub fn saturation(c: RGB) -> f64 {
    let maximum = f64::max(f64::max(c.r as f64 / 255.0, c.g as f64 / 255.0), c.b as f64 / 255.0);
    let minimum = f64::min(f64::min(c.r as f64 / 255.0, c.g as f64 / 255.0), c.b as f64 / 255.0);


    if maximum == minimum {
        return 0.0;
    }

    let l = (maximum + minimum) / 2.0;
    let d = maximum - minimum;

    if l > 0.5 {
        d / (2.0 - maximum - minimum)
    } else {
        d / (maximum + minimum)
    }
}

pub fn importance(crop: &Crop, x: u32, y: u32) -> f64 {
    if crop.x > x || x >= crop.x + crop.width || crop.y > y || y >= crop.y + crop.height {
        return OUTSIDE_IMPORTANCE;
    }

    let xf = (x - crop.x) as f64 / (crop.width as f64);
    let yf = (y - crop.y) as f64 / (crop.height as f64);

    let px = (0.5 - xf).abs() * 2.0;
    let py = (0.5 - yf).abs() * 2.0;

    let dx = f64::max(px - 1.0 + EDGE_RADIUS, 0.0);
    let dy = f64::max(py - 1.0 + EDGE_RADIUS, 0.0);
    let d = (dx * dx + dy * dy) * EDGE_WEIGHT;

    let mut s = 1.41 - (px * px + py * py).sqrt();
    if RULE_OF_THIRDS {
        s += (f64::max(0.0, s + d + 0.5) * 1.2) * (thirds(px) + thirds(py))
    }

    s + d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gray(c: u8) -> RGB {
        RGB::new(c, c, c)
    }

    #[test]
    fn chop_test() {
        assert_eq!(1.0, chop(1.1));
        assert_eq!(-1.0, chop(-1.1));
    }

    #[test]
    fn thirds_test() {
        assert_eq!(0.0, thirds(0.0));
    }

    #[test]
    fn bounds_test() {
        assert_eq!(0, bounds(-1.0));
        assert_eq!(0, bounds(0.0));
        assert_eq!(10, bounds(10.0));
        assert_eq!(255, bounds(255.0));
        assert_eq!(255, bounds(255.1));
    }

    #[test]
    fn cie_test() {
        assert_eq!(0.0, cie(gray(0)));
        assert_eq!(331.49999999999994, cie(gray(255)));
    }

    #[test]
    fn skin_col_test() {
        assert!(skin_col(gray(0)).is_nan());
        assert_eq!(0.7550795306611965, skin_col(gray(255)));
    }

    #[test]
    fn saturation_tests() {
        assert_eq!(0.0, saturation(gray(0)));
        assert_eq!(0.0, saturation(gray(255)));
        assert_eq!(1.0, saturation(RGB::new(255, 0, 0)));
        assert_eq!(1.0, saturation(RGB::new(0, 255, 0)));
        assert_eq!(1.0, saturation(RGB::new(0, 0, 255)));
        assert_eq!(1.0, saturation(RGB::new(0, 255, 255)));
    }

    #[test]
    fn importance_tests() {
        assert_eq!(
            -6.404213562373096,
            importance(
                &Crop { x: 0, y: 0, width: 1, height: 1 },
                0,
                0)
        );
    }

    #[test]
    fn crop_scale_test() {
        let crop = Crop{
            x:2,
            y:4,
            width:8,
            height:16
        };

        let scaled_crop = crop.scale(0.5);

        assert_eq!(1, scaled_crop.x);
        assert_eq!(2, scaled_crop.y);
        assert_eq!(4, scaled_crop.width);
        assert_eq!(8, scaled_crop.height);
    }


    fn any_score() -> Score {
        Score { detail: 1.0, saturation: 2.0, skin: 3.0, total: 6.0 }
    }

}