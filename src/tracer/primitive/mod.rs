pub mod plane;
pub mod sphere;
pub mod triangle;

use bvh::ray::{Ray, Intersection};
use bvh::bounding_hierarchy::BHShape;
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

pub struct HitData {
    pub material: Material,
    pub normal: Vector3<f32>,
}

pub trait Primitive : BHShape {
    fn get_hit_data(&self, intersection: &Intersection) -> HitData; 
}
