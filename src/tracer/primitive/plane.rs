use nalgebra::{Vector3, Point3};
use bvh::ray::Ray;
use std::f32;
use tracer::primitive::{Material, Intersection, Primitive};

#[derive(Debug)]
pub struct Plane {
    pub p0: Point3<f32>,
    pub material: Material,
    pub normal: Vector3<f32>,
}

/*impl Primitive for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        // TODO: not sure why I take the negative here. but it works?
        let denom = -self.normal.dot(&ray.direction);
        let lol = 1. / denom;
        if denom > 1e-5 {
            let p0l0 = self.p0 - ray.origin;
            let distance = -p0l0.dot(&self.normal) * lol;
            if distance >= 0.0 {
                Some(Intersection {
                    material: self.material.clone(),
                    normal: self.normal,
                    distance,
                    intersection: ray.origin + (distance * ray.direction),
                    depth: 0,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}*/

