use cgmath::{Vector3, Point3};
use cgmath::InnerSpace;
use tracer::ray::Ray;
use tracer::ray::Ray2;
use std::f32;
use tracer::primitive::{Material, Intersection, Primitive};
use stdsimd::simd::f32x4;
use stdsimd::vendor;
use vec::{dot,cross, vec_to_f32x4, pnt_to_f32x4, f32x4_to_pnt, f32x4_to_vec};

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub p0: Point3<f32>,
    pub p1: Point3<f32>,
    pub p2: Point3<f32>,
    pub material: Material,
    pub normal: Vector3<f32>,
}


struct Triangle2 {
    p0: f32x4,
    p1: f32x4,
    p2: f32x4,
    material: Material,
    normal: Vector3<f32>,
}

// hand-rolled SIMD
fn intersect(
    p0: f32x4,
    p1: f32x4,
    p2: f32x4,
    origin: f32x4,
    direction: f32x4,
    normal: Vector3<f32>,
    material: Material,
) -> Option<Intersection> {
    let e1 = p1 - p0;
    let e2 = p2 - p0;
    let p = cross(direction, e2);
    let det = dot(e1, p);
    let det1 = det.extract(0);
    if let Material::Conductor{..} = material {
       if det1 < f32::EPSILON {
        return None
       }
    }
    if (det1 > -f32::EPSILON && det1 < f32::EPSILON) {
        return None;
    }
    let inv_det = unsafe { vendor::_mm_rcp_ps(det) };
    let t = origin - p0;
    let u = dot(t, p) * inv_det;
    let u = u.extract(0);
    if u < 0. || u > 1. {
        return None;
    }
    let q = cross(t, e1);
    let v = dot(direction, q) * inv_det;
    let v = v.extract(0);
    if v < 0. || u + v > 1. {
        return None;
    }
    let t = dot(e2, q) * inv_det;

    // TODO simd this
    if t.extract(0) <= f32::EPSILON {
        return None;
    }
    let intersection = origin + t * direction;
    Some(Intersection {
        intersection: f32x4_to_pnt(intersection),
        normal: normal,
        distance: t.extract(0),
        material: material,
    })
}

impl Primitive for Triangle {
    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        intersect(
            pnt_to_f32x4(self.p0),
            pnt_to_f32x4(self.p1),
            pnt_to_f32x4(self.p2),
            pnt_to_f32x4(ray.origin),
            vec_to_f32x4(ray.direction),
            self.normal,
            self.material,
        )
        /*
        let e1 = self.p1 - self.p0;
        let e2 = self.p2 - self.p0;
        let p = ray.direction.cross(e2);
        let det = e1.dot(p);

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
        let u = t.dot(p) * inv_det;
        if u < 0. || u > 1. { return None }
        let q = t.cross(e1);
        let v = ray.direction.dot(q) * inv_det;
        if v < 0. || u + v > 1. { return None }
        let t = e2.dot(q) * inv_det;


        if t > f32::EPSILON {
        let intersection = ray.origin + t * ray.direction;
            Some(Intersection {
                intersection: intersection,
                normal: self.normal,
                material: self.material,
                distance: t,
            })
        } else {
            None
        }*/

    }
}
