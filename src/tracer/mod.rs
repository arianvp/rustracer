pub mod camera;
pub mod primitive;
pub mod ray;
pub mod scene;

use cgmath::{EuclideanSpace, InnerSpace};
use cgmath::{Vector3, Point3};
use std::f32;
use std::cmp::Ordering;


use half::f16;
use self::scene::Scene;
use self::ray::Ray;
use self::camera::Camera;
use self::primitive::{Light, Primitive, Material, Intersection};


pub fn tracer(camera: &Camera, scene: &Scene, buffer: &mut Vec<[f16; 4]>) {
    for y in 0..camera.width {
        for x in 0..camera.height {
            let color = trace(scene, camera.generate(x,y));
            let idx = x + y * camera.height;
            for i in 0..2 {
                buffer[idx][i] = f16::from_f32(color[i]);
            }

        }
    }

}

fn nearest_intersection(scene: &Scene, ray: Ray) -> Option<Intersection> {
    scene
        .primitives
        .iter()
        .filter_map(|p| p.intersect(&ray))
        .min_by(|a, b| {
            a.distance.partial_cmp(&b.distance).unwrap_or(
                Ordering::Equal,
            )
        })
}

fn direct_illumination(scene: &Scene, intersection: Intersection) -> Vector3<f32> {
    scene.lights.iter().fold(
        Vector3::new(0.0, 0.0, 0.0),
        |color, light| {
            color + intersection.brdf(light)
        },
    )
}

fn trace(scene: &Scene, ray: Ray) -> Vector3<f32> {
    nearest_intersection(scene, ray)
        .map(|i| direct_illumination(scene, i))
        .unwrap_or(Vector3::new(0.0, 0.0, 0.0))
}
