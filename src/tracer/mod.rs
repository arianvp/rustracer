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

//use stdsimd::simd::u8x16;
//use stdsimd::vendor::_mm_adds_epu8;


// TODO: clamp or bugs will be
pub fn tracer(camera: &Camera, scene: &Scene, buffer: &mut Vec<[f16; 4]>) {
    for y in 0..camera.width {
        for x in 0..camera.height {
            let ray = camera.generate(x, y);
            let color = trace(scene, ray);
            let idx = x + y * camera.height;
            buffer[idx][0] = f16::from_f32(color[0]);
            buffer[idx][1] = f16::from_f32(color[1]);
            buffer[idx][2] = f16::from_f32(color[2]);
            // TODO we can use the saturated-add instruction here
            /*
             buffer[idx][0] = u8::max(0,u16::min(256, f32::floor(color[0] * 256.0) as u16) as u8);
            buffer[idx][1] = u8::max(0,u16::min(256, f32::floor(color[1] * 256.0) as u16) as u8);
            buffer[idx][2] = u8::max(0,u16::min(256, f32::floor(color[2] * 256.0) as u16) as u8);*/

        }
    }

}

// what a beauty
fn nearest_intersection(scene: &Scene, ray: Ray) -> Option<Intersection> {
    scene
        .primitives
        .iter()
        .filter_map(|circle| circle.intersect(&ray))
        .min_by(|a, b| {
            a.distance.partial_cmp(&b.distance).unwrap_or(
                Ordering::Equal,
            )
        })
}

fn direct_illumination(
    scene: &Scene,
    ray_direction: Vector3<f32>,
    intersection: Intersection,
) -> Vector3<f32> {
    scene.lights.iter().fold(
        Vector3::new(0.0, 0.0, 0.0),
        |color, light| {
            // simple lambertian surface
            let light_direction = light.position - intersection.intersection;
            let light_distance = light_direction.magnitude();
            let light_direction = light_direction.normalize();

            // take light distance into consideration

            let l_dot_n = f32::max(0.0, light_direction.dot(intersection.normal));


            0.01 * l_dot_n * intersection.material.color
        },
    )
}

fn trace(scene: &Scene, ray: Ray) -> Vector3<f32> {
    nearest_intersection(scene, ray)
        .map(|i| direct_illumination(scene, ray.direction, i))
        .unwrap_or(Vector3::new(0.0, 0.0, 0.0))
}
