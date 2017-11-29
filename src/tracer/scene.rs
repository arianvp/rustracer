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
                /*Light {
                    intensity: 2.0,
                    position: Point3::new(0.0, -2.0, 0.0),
                },*/
            ],
            primitives: vec![
                Primitive::Plane{
                    p0: Point3::new(0.0, 0.0, 0.0),
                    normal: Vector3::new(0.0, 1.0, 0.0),
                    material: Material { color: Vector3::new(0.0, 1.0, 0.0) },
                },
                Primitive::Sphere {
                    material: Material { color: Vector3::new(1.0, 0.0, 0.0) },
                    position: Point3::new(0.0, 1.0, 0.0),
                    radius: 0.5,
                },
                Primitive::Sphere {
                    material: Material { color: Vector3::new(1.0, 1.0, 0.0) },
                    position: Point3::new(1.0, 1.0, 0.0),
                    radius: 0.3,
                },
            ],
        }
    }
}
