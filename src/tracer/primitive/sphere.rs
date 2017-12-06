use cgmath::{Point3, InnerSpace};
use std::mem;
use tracer::primitive::{Material, Intersection, Primitive};
use tracer::ray::Ray;

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub position: Point3<f32>,
    pub radius: f32,
    pub material: Material,
}

impl Primitive for Sphere {
    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        let distance = self.position - ray.origin;
        let tca = distance.dot(ray.direction);
        if tca < 0.0 {
            return None;
        }
        let d2 = distance.dot(distance) - tca * tca;
        let r2 = self.radius * self.radius;
        if d2 > r2 {
            return None;
        }
        let thc = (r2 - d2).sqrt();
        let mut t0 = tca - thc;
        let mut t1 = tca + thc;
        if t0 > t1 {
            mem::swap(&mut t0, &mut t1)
        }
        if t0 < 0.0 {
            t0 = t1;
            if t0 < 0.0 {
                return None;
            }
        }
        let intersection = ray.origin + ray.direction * t0;
        let normal = (intersection - self.position).normalize();
        Some(Intersection {
            material: self.material,
            normal,
            intersection: intersection,
            distance: t0,
        })
        /*let intersection = ray.origin + ray.direction * t0;
        let normal = (intersection - position).normalize();
        let normal = if inside { -normal} else { normal };

        let m = ray.origin - self.position;
        let b = m.dot(ray.direction);
        let c = m.dot(m) - (self.radius * self.radius);
        if c > 0.0 && b > 0.0 {
            return None
        }
        let d = b * b - c;
        if d < 0.0 {
            return None
        }
        let mut normal_mult = 1.0;
        let mut t = -b - d.sqrt();
        if t < 0.0 {
            t = -b + d.sqrt();
            normal_mult = -1.0;
        }
        let intersection = ray.origin + ray.direction*t;
        let normal = (intersection - self.position).normalize(); // * normal_mult;


        Some(Intersection {
            material: self.material,
            normal,
            intersection: intersection,
            distance: t,
        })*/
    }
}