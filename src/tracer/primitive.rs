use cgmath::{Vector3, Point3};
use cgmath::{EuclideanSpace, InnerSpace};
use super::ray::Ray;
use std::f32;
use std::mem;


#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub color: Vector3<f32>,
    pub spec: f32,
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

pub enum Primitive {
    Sphere {
        position: Point3<f32>,
        radius: f32,
        material: Material,
    },
    Plane {
        normal: Vector3<f32>,
        p0: Point3<f32>,
        material: Material,
    },
}

impl Primitive {
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            &Primitive::Plane {
                normal,
                p0,
                material,
            } => {
                // TODO: not sure why I take the negative here. but it works?
                let denom = -normal.dot(ray.direction);
                if denom > 1e-5 {
                    let p0l0 = p0 - ray.origin;
                    let distance = -p0l0.dot(normal) / denom;
                    if distance >= 0.0 {
                        Some(Intersection {
                            material,
                            normal,
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
            &Primitive::Sphere {
                position,
                radius,
                material,
            } => {
                let distance = position - ray.origin;
                let tca = distance.dot(ray.direction);
                if tca < 0.0 {
                    return None;
                }
                let d2 = distance.dot(distance) - tca * tca;
                if d2 > radius * radius {
                    return None;
                }
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
                    if t0 < 0.0 {
                        return None;
                    }
                }
                let distance = t0;
                let intersection = ray.origin + ray.direction * t0;
                let normal = (intersection - position).normalize();
                Some(Intersection {
                    material,
                    normal,
                    // NOTE: I added a little bit of Bias, or we would reintersect the ball
                    intersection: intersection + normal * 1e-4,
                    distance,
                })
            }
        }
    }
}
