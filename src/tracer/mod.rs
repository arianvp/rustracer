pub mod camera;
pub mod primitive;
pub mod ray;
pub mod scene;

use cgmath::{EuclideanSpace, InnerSpace, Array};
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
            let color = trace(scene, camera.generate(x, y), 5);
            let idx = x + y * camera.height;
            for i in 0..3 {
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
pub fn brdf(i: &Intersection, light: &Light) -> f32 {
    let light_direction = light.position - i.intersection;
    let light_distance = light_direction.magnitude();
    let light_direction = light_direction.normalize();
    let l_dot_n = f32::max(0.0, light_direction.dot(i.normal));
    (light.intensity * l_dot_n) / (light_distance * light_distance)
}

// Actually not dependend on material
fn direct_illumination(scene: &Scene, intersection: Intersection) -> Vector3<f32> {
    scene.lights.iter().fold(
        Vector3::new(0.0, 0.0, 0.0),
        |color, light| {
            let origin = intersection.intersection;
            let direction = (light.position - intersection.intersection).normalize();
            let ray = Ray { origin, direction };
            let to_mul = if nearest_intersection(scene, ray).is_some() {
                Vector3::new(0.0, 0.0, 0.0)
            } else {
                brdf(&intersection, light) * intersection.material.color
            };

            color + to_mul
        },
    )
}

fn reflect(direction: Vector3<f32>, normal: Vector3<f32>) -> Vector3<f32> {
    (direction - 2. * direction.dot(normal) * normal)
}

fn schlick(direction: Vector3<f32>, normal: Vector3<f32>, n1: f32, n2: f32) -> f32 {
    let i = (n1 - n2) / (n1 + n2);
    let r0 = i * i;
    r0 + (1.0 - r0) * (1.0 - -direction.dot(normal)).powi(5)
}

fn trace(scene: &Scene, ray: Ray, depth: u32) -> Vector3<f32> {
    let mut ray = ray.clone();
    let mut a = 1.0;
    let mut color = Vector3::new(0.0,0.0,0.0);
    for _ in 0..depth {
        match nearest_intersection(scene, ray) {
            None => {
                break;
            },
            Some(i) => {
                let s = i.material.spec;
                let d = 1.0 - s;
                color += a*d*direct_illumination(scene, i);
                a *= s;
                ray = Ray {
                    origin: i.intersection,
                    direction: reflect(ray.direction, i.normal),
                }
            },
        }
    }
    color
}
