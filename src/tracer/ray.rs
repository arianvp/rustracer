extern crate cgmath;

use cgmath::{Vector3, Point3};
use std::f32;
use stdsimd::simd::f32x4;
use vec::Vec3x4;



#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}

#[derive(Debug, Copy, Clone)]
pub struct Ray4 {
    pub origin: Vec3x4,
    pub direction: Vec3x4,
}

