use nalgebra::{Vector3, Point3};
use bvh::ray::Ray;
use std::f32;
use tracer::primitive::{Material, Intersection, Primitive};
use stdsimd::simd::f32x4;
use stdsimd::vendor;
use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::{BoundingHierarchy, BHShape};


#[derive(Debug)]
pub struct Triangle {
    pub p0: Point3<f32>,
    pub p1: Point3<f32>,
    pub p2: Point3<f32>,
    pub material: Material,
    pub n0: Vector3<f32>,
    pub n1: Vector3<f32>,
    pub n2: Vector3<f32>,
    pub node_index: usize,
}

impl Bounded for Triangle {
    fn aabb(&self) -> AABB {
        AABB::empty().grow(&self.p0).grow(&self.p1).grow(&self.p2)
    }
}


impl BHShape for Triangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_index
    }
}


impl Primitive for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let e1 = self.p1 - self.p0;
        let e2 = self.p2 - self.p0;
        let p = ray.direction.cross(&e2);
        let det = e1.dot(&p);

        // backface culling
        if let Material::Conductor{..} = self.material {
            if det < f32::EPSILON {
                return None
            }
        }

        if (det > -f32::EPSILON && det < f32::EPSILON) {
            return None
        }

        let inv_det = 1.0 / det;
        let t = ray.origin - self.p0;
        let u = t.dot(&p) * inv_det;
        if u < 0. || u > 1. { return None }
        let q = t.cross(&e1);
        let v = ray.direction.dot(&q) * inv_det;
        if v < 0. || u + v > 1. { return None }
        let t = e2.dot(&q) * inv_det;


        if t > f32::EPSILON {
        let intersection = ray.origin + t * ray.direction;
            Some(Intersection {
                intersection: intersection,
                normal: ((1. - u - v) * self.n0 + u * self.n1 + v * self.n2).normalize(),
                // TODO remove material from Intersection
                material: self.material.clone(),
                distance: t,
            })
        } else {
            None
        }

    }
}
