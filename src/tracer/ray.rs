extern crate cgmath;

use cgmath::{Vector3, Point3};
use std::f32;
use stdsimd::simd::f32x4;



#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}

#[derive(Debug, Copy, Clone)]
pub struct Ray2 {
    pub origin: f32x4,
    pub direction: f32x4,
}

