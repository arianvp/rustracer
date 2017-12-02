extern crate cgmath;

use cgmath::{Vector3, Point3};
use std::f32;



#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}



