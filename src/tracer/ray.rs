extern crate cgmath;

use cgmath::{Vector3, Point3};
use std::f32;
use simd::f32x4;



#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}

#[derive(Debug, Copy, Clone)]
// TODO, this probably should not be a copy
pub struct Ray4 {
    pub origin_x: f32x4,
    pub origin_y: f32x4,
    pub origin_z: f32x4,

    pub direction_x: f32x4,
    pub direction_y: f32x4,
    pub direction_z: f32x4,
}


