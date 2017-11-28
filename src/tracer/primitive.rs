use cgmath::{Vector3, Point3};
use cgmath::{EuclideanSpace, InnerSpace};
use super::ray::Ray;
use std::f32;

#[derive(Debug, Copy, Clone)]
pub struct Intersection {
    pub distance: f32, // here for convenience
    pub intersection: Point3<f32>,
    pub normal: Vector3<f32>,
    pub material: Material,
}

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub color: Vector3<f32>,
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
        distance: f32,
        material: Material,
    }
}

impl Primitive {
    pub fn plane(p1: Point3<f32>, p2: Point3<f32>, p3: Point3<f32>, material: Material)  -> Primitive {
        let normal = (p3 - p1).cross(p2 - p1).normalize();
        Primitive::Plane{
            normal,
            distance: -1.0*normal.dot(p1 - Point3::new(0.0,0.0,0.0)),
            material,
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        match self {
            &Primitive::Plane{normal, distance, material} => {
                //let distance = self.position - ray.origin;
                let denom = ray.direction.dot(normal);
                if denom < f32::EPSILON {
                    None
                } else {
                    let t = (ray.origin.dot(normal) + distance) / ray.direction.dot(normal);
                    let intersection = ray.origin + t * ray.direction;

                    let distance = (intersection - ray.origin).magnitude();
                    if t >= 0.0 {
                        Some(Intersection{
                            distance,
                            intersection,
                            normal,
                            material,
                        })
                    } else {
                        None
                    }
                }
            },
            &Primitive::Sphere{ position, radius, material } => {
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
                let t0 = tca - thc;
                let t1 = tca + thc;
                if t0 >= 0.0 {
                    let distance = t0;
                    let intersection = ray.origin + ray.direction * t0;
                    let normal = intersection - position;
                    Some(Intersection {
                        material,
                        normal,
                        intersection,
                        distance,
                    })
                } else if t1 >= 0.0 {
                    let distance = t1;
                    let intersection = ray.origin + ray.direction * t1;
                    let normal = intersection - position;
                    Some(Intersection { material, normal, intersection, distance})
                } else {
                    None
                }
            },
        }
    }
}

