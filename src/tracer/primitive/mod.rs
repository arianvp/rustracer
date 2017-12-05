pub mod plane;
pub mod sphere;
pub mod triangle;

use super::ray::Ray;
use cgmath::{Vector3, Point3};

#[derive(Debug, Copy, Clone)]
pub enum Material { 
    Conductor {
        color: Vector3<f32>,
        spec: f32,
    },
    Dielectric {
        absorbance: Vector3<f32>,
        n1: f32,  // TODO: we should just have 1 n, and keep track in the tracer what the transitions are
        n2: f32,

    }
}


#[derive(Debug, Copy, Clone)]
pub struct Light {
    pub position: Point3<f32>,
    pub intensity: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct Intersection {
    pub distance: f32, // here for convenience
    pub intersection: Point3<f32>,
    pub normal: Vector3<f32>,
    pub material: Material,
}

pub trait Primitive {
    fn intersect(&self, ray: Ray) -> Option<Intersection>;
}
