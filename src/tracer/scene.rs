use cgmath::{Vector3, Point3};
use super::primitive::{Light, Primitive, Material};

pub struct Scene {
    pub primitives: Vec<Primitive>,
    pub lights: Vec<Light>,
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
            ],
            primitives: vec![
                Primitive::Plane{
                    p0: Point3::new(0.0, 0.0, 0.0),
                    normal: Vector3::new(0.0, 1.0, 0.0),
                    material: Material{ spec: 0.96, color: Vector3::new(0.0, 1.0, 0.0) },
                },
                Primitive::Sphere {
                    material: Material{ spec: 1.0, color: Vector3::new(1.0, 0.0, 0.0) },
                    position: Point3::new(0.0, 1.0, 0.0),
                    radius: 0.5,
                },
                Primitive::Sphere {
                    material: Material{ spec: 0.0, color: Vector3::new(0.0, 0.0, 1.0) },
                    position: Point3::new(1.0, 1.0, 0.0),
                    radius: 0.3,
                },
            ],
        }
    }
}