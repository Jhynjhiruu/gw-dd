use std::fmt::Display;

use binrw::binrw;

#[binrw]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Vec3 {
    pub const ZERO: Self = Vec3::new(0.0, 0.0, 0.0);

    pub const X: Self = Vec3::new(1.0, 0.0, 0.0);
    pub const Y: Self = Vec3::new(0.0, 1.0, 0.0);
    pub const Z: Self = Vec3::new(0.0, 0.0, 1.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}
