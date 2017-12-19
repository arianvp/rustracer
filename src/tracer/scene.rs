use nalgebra::{Vector3, Point3};
use super::primitive::{Light, Primitive, Material, Intersection};
use super::primitive::plane::Plane;
use super::primitive::sphere::Sphere;
use super::primitive::triangle::Triangle;
use bvh::ray::Ray;
use std::cmp::Ordering;
use super::mesh::Mesh;

use std::path::Path;

pub struct Scene {
    pub lights: Vec<Light>,
    pub planes: Vec<Plane>,
    pub spheres: Vec<Sphere>,
    pub meshes: Vec<Mesh>,
}


#[inline]
fn nearest_intersection_<T: Primitive>(primitives: &[T], ray: &Ray) -> Option<Intersection> {
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
        let mesh1 = Mesh::load_from_path2(
            &Path::new("./assets/sponza.obj"),
            /*Vector3::new(-0.7, 1.3, 1.1),
            0.5,
            Material::Conductor {
                spec: 0.0,
                color: Vector3::new(0.8, 0.8, 0.8),
            },*/
        );
        let mesh2 = Mesh::load_from_path(
            &Path::new("./assets/cube.obj"),
            Vector3::new(0.5, 1.0, 1.0),
            0.4,
            Material::Dielectric {
                n1: 1.0,
                n2: 1.21,
                absorbance: Vector3::new(0.2, 3.0, 3.0),
            },
        ).expect("Error loading");
       // let mut triangles = mesh1.triangles;
       // triangles.extend(mesh2.triangles);
        Scene {
            meshes: vec![mesh1],
            lights: vec![
                Light {
                    intensity: 12.0,
                    position: Point3::new(0.0, 1.0, 0.0),
                },
                Light {
                    intensity: 5.0,
                    position: Point3::new(1.0, 3.0, -4.0),
                },
            ],
            planes: vec![
                Plane {
                    p0: Point3::new(0.0, 0.0, 0.0),
                    normal: Vector3::new(0.0, 1.0, 0.0),
                    material: Material::Conductor {
                        spec: 0.0,
                        color: Vector3::new(0.3, 1.0, 0.3),
                    },
                },
                /*Plane {
                    p0: Point3::new(0.0, 40.0, 0.0),
                    normal: Vector3::new(0.0, -1.0, 0.0),
                    material: Material::Conductor {
                        spec: 0.0,
                        color: Vector3::new(0.3, 1.0, 0.3),
                    },
                },*/
            ],
            spheres: vec![
                /********Sphere {
                    material: Material::Conductor {
                        spec: 0.3,
                        color: Vector3::new(1.0, 0.0, 0.3),
                    },
                    position: Point3::new(0.0, 1.0, 0.0),
                    radius: 0.5,
                },
                Sphere {
                    material: Material::Dielectric {
                        n1: 1.0,
                        n2: 1.21,
                        absorbance: Vector3::new(0.7, 4.0, 0.2),
                    },
                    position: Point3::new(3.0, 1.0, 0.0),
                    radius: 1.0,
                },*/
            ],
        }
    }

    pub fn nearest_intersection(&self, ray: &Ray) -> Option<Intersection> {
        // we iterate over each of the primitives together for more cache coherence
        let plane = nearest_intersection_(&self.planes, ray);
        let sphere = nearest_intersection_(&self.spheres, ray);
        let mesh = nearest_intersection_(&self.meshes, ray);

        let mut nearest = None;
        for y in [plane, sphere, mesh].iter() {
            if let &Some(ref i) = y {
                let r: &mut Intersection = nearest.get_or_insert(i.clone());
                if i.clone().distance < r.distance {
                    *r = i.clone()
                }
            }
        }
        nearest
    }
}
