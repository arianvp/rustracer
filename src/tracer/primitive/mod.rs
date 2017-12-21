pub mod plane;
pub mod sphere;
pub mod triangle;

use bvh::ray::Ray;
use nalgebra::{Vector3, Point3};

#[derive(Debug, Clone)]
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


#[derive(Debug, Clone)]
pub struct Light {
    pub position: Point3<f32>,
    pub intensity: f32,
}

#[derive(Debug, Clone)]
pub struct Intersection {
    pub distance: f32, // here for convenience
    pub intersection: Point3<f32>,
    pub normal: Vector3<f32>,
    pub material: Material,
    pub depth: u32,
}

pub trait Primitive {
    fn intersect(&self, ray: &Ray) ->  Option<Intersection>;
}
