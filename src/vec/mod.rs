/*use stdsimd::simd::f32x4;
use stdsimd::vendor;
use cgmath::{Vector3, Point3};
use cgmath::EuclideanSpace;


#[derive(Debug, Copy, Clone)]
pub struct Vec3x4 {
    x4: f32x4,
    y4: f32x4,
    z4: f32x4,
}

pub struct Color4 {
    r4: f32x4,
    g4: f32x4,
    b4: f32x4,
}

#[inline]
pub fn vec_to_f32x4(x: Vector3<f32>) -> f32x4 {
    f32x4::new(x.x, x.y, x.z, 0.0)
}

#[inline]
pub fn pnt_to_f32x4(x: Point3<f32>) -> f32x4 {
    f32x4::new(x.x, x.y, x.z, 0.0)
}

#[inline]
pub fn f32x4_to_vec(x: f32x4) -> Vector3<f32> {
    Vector3::new(x.extract(0), x.extract(1), x.extract(2))
}

#[inline]
pub fn f32x4_to_pnt(x: f32x4) -> Point3<f32> {
    Point3::new(x.extract(0), x.extract(1), x.extract(2))
}

#[inline]
pub fn dot(x: f32x4, y: f32x4) -> f32x4 {
    unsafe {
        vendor::_mm_dp_ps(x, y, 0xFF)
    }
}

#[inline]
pub fn cross(a: f32x4, b: f32x4) -> f32x4 {
    unsafe {
      vendor::_mm_sub_ps(
        vendor::_mm_mul_ps(vendor::_mm_shuffle_ps(a, a, 0xc9), vendor::_mm_shuffle_ps(b, b, 0xd2)), 
        vendor::_mm_mul_ps(vendor::_mm_shuffle_ps(a, a, 0xd2), vendor::_mm_shuffle_ps(b, b, 0xc9)))
    }
}
*/
