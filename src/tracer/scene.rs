use cgmath::{Vector3, Point3};
use super::primitive::{Light, Primitive, Material, Sphere, Plane, Intersection};
use super::ray::{Ray};
use std::cmp::Ordering;

pub struct Scene {
    pub lights: Vec<Light>,
    pub planes: Vec<Plane>,
    pub spheres: Vec<Sphere>,
}

fn nearest_intersection_<T: Primitive>(primitives: &[T], ray: Ray) -> Option<Intersection> {
    primitives.iter().filter_map(|p| p.intersect(ray)).min_by(
        |a, b| {
            a.distance.partial_cmp(&b.distance).unwrap_or(
                Ordering::Equal,
            )
        },
    )
}


impl Scene {
    pub fn new() -> Scene {
        Scene {
            lights: vec![
                Light {
                    intensity: 1.0,
                    position: Point3::new(0.0, 3.0, 0.0),
                },
                Light {
                    intensity: 9.0,
                    position: Point3::new(7.0, 8.0, 0.0),
                },
                Light {
                    intensity: 0.3,
                    position: Point3::new(0.0, 1.0, 0.0),
                },
            ],
            planes: vec![
                Plane{
                    p0: Point3::new(0.0, 0.0, 0.0),
                    normal: Vector3::new(0.0, 1.0, 0.0),
                    material: Material::Conductor{ spec: 0.0, color: Vector3::new(0.0, 1.0, 0.0) },
                },
            ],
            spheres: vec![
                Sphere {
                    material: Material::Conductor{ spec: 0.3, color: Vector3::new(1.0, 0.0, 0.0) },
                    position: Point3::new(0.0, 1.0, 0.0),
                    radius: 0.5,
                },
                Sphere {
                    material: Material::Conductor{ spec: 0.0, color: Vector3::new(1.0, 0.0, 1.0) },
                    position: Point3::new(-0.5, 1.0, 1.0),
                    radius: 0.5,
                },
                Sphere {
                    material: Material::Dielectric{ n1: 1.0, n2: 1.51, absorbance: Vector3::new(0.0, 0.0, 0.0) },
                    position: Point3::new(5.0, 1.0, 0.0),
                    radius: 1.0,
                },
            ],
        }
    }

    pub fn nearest_intersection(&self, ray: Ray) -> Option<Intersection> {
        // we iterate over each of the primitives together for more cache coherence
        let plane = nearest_intersection_(&self.planes, ray);
        let sphere = nearest_intersection_(&self.spheres, ray);

        match (plane, sphere) {
            (Some(plane), Some(sphere)) => if plane.distance < sphere.distance { Some(plane) } else { Some(sphere) },
            (Some(plane), None) => Some(plane),
            (None, Some(sphere)) => Some(sphere),
            (None, None) => None,
        }
    }
}
