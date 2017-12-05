use cgmath::{Vector3, Point3};
use cgmath::InnerSpace;
use tracer::ray::Ray;
use std::f32;
use tracer::primitive::{Material, Intersection, Primitive};

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub p0: Point3<f32>,
    pub p1: Point3<f32>,
    pub p2: Point3<f32>,
    pub material: Material,
    pub normal: Vector3<f32>,
}

impl Primitive for Triangle {
    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        let e1 = self.p1 - self.p0;
        let e2 = self.p2 - self.p0;
        let normal = e1.cross(e2).normalize();
        let p = ray.direction.cross(e2);
        let det = e1.dot(p);


        if (det > -f32::EPSILON && det < f32::EPSILON) {
            return None
        }

        let inv_det = 1.0 / det;
        let t = ray.origin - self.p0;
        let u = t.dot(p) * inv_det;
        if u < 0. || u > 1. { return None }
        let q = t.cross(e1);
        let v = ray.direction.dot(q) * inv_det;
        if v < 0. || u + v > 1. { return None }
        let t = e2.dot(q) * inv_det;

        //println!("normal: {:?}", normal);

        if t > f32::EPSILON {
        let intersection = ray.origin + t * ray.direction;
            Some(Intersection {
                intersection: intersection,
                normal,
                material: self.material,
                distance: t,
            })
        } else {
            None
        }

    }
}
