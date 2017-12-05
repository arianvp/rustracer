use cgmath::{Vector3, Point3};
use cgmath::{EuclideanSpace, InnerSpace};
use super::ray::Ray;
use std::f32;
use std::mem;



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
pub struct Intersection {
    pub distance: f32, // here for convenience
    pub intersection: Point3<f32>,
    pub normal: Vector3<f32>,
    pub material: Material,
}



#[derive(Debug, Copy, Clone)]
pub struct Light {
    pub position: Point3<f32>,
    pub intensity: f32,
}


#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub position: Point3<f32>,
    pub radius: f32,
    pub material: Material,
}

#[derive(Debug, Copy, Clone)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub p0: Point3<f32>,
    pub material: Material,
}

pub trait Primitive {
    fn intersect(&self, ray: Ray) -> Option<Intersection>;
}

impl Primitive for Plane {
    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        // TODO: not sure why I take the negative here. but it works?
        let denom = -self.normal.dot(ray.direction);
        if denom > 1e-5 {
            let p0l0 = self.p0 - ray.origin;
            let distance = -p0l0.dot(self.normal) / denom;
            if distance >= 0.0 {
                Some(Intersection {
                    material: self.material,
                    normal: self.normal,
                    distance,
                    intersection: ray.origin + (distance * ray.direction),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Primitive for Sphere {
    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        /*let distance = position - ray.origin;
        let tca = distance.dot(ray.direction);
        if tca < 0.0 {
            return None;
        }
        let d2 = distance.dot(distance) - tca * tca;
        if d2 > radius * radius {
            return None;
        }
        let mut inside = false;
        let thc = (radius * radius - d2).sqrt();
        let mut t0 = tca - thc;
        let mut t1 = tca + thc;
        // NOTE: used swap trick from Scratchapixel. It actually gives us a bit better
        // frame rate! 
        // https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/minimal-ray-tracer-rendering-spheres
        if t0 > t1 {
            mem::swap(&mut t0, &mut t1)
        }
        if t0 < 0.0 {
            t0 = t1;
            inside = true;
            if t0 < 0.0 {
                return None;
            }
        }
        let distance = t0;
        let intersection = ray.origin + ray.direction * t0;
        let normal = (intersection - position).normalize();
        let normal = if inside { -normal} else { normal };*/

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
        let normal = (intersection - self.position).normalize() * normal_mult;


        Some(Intersection {
            material: self.material,
            normal,
            intersection: intersection,
            distance: t,
        })
    }
}
