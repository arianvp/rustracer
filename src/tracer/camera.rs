use cgmath::{Vector3, Point3};
use cgmath::InnerSpace;
use cgmath::ElementWise;
use tracer::ray::Ray;
use std::f32;

pub struct Camera {
    origin: Point3<f32>,
    target: Point3<f32>,
    focal_distance: f32,
    direction: Vector3<f32>,

    // screen plane
    p1: Point3<f32>,
    p2: Point3<f32>,
    p3: Point3<f32>,

    up: Vector3<f32>,
    right: Vector3<f32>,
    width: usize,
    height: usize,
    lens_size: f32,

    depth: u32,
}

impl Camera {
  pub fn generate(&self, x: usize, y: usize) -> Ray {
      let u = (x as f32) / (self.width as f32);
      let v = (y as f32) / (self.height as f32);
      let target = self.p1 + u * (self.p2 - self.p1) + v * (self.p3 - self.p1);
      let direction = (target - self.origin).normalize();
      Ray::new(self.origin, direction, f32::INFINITY)
  }
}
