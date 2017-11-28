pub mod camera;
pub mod primitive;
pub mod ray;
pub mod scene;

use cgmath::{EuclideanSpace, InnerSpace};
use cgmath::{Vector3, Point3};
use std::f32;
use std::cmp::Ordering;


use self::scene::Scene;
use self::ray::Ray;
use self::camera::Camera;
use self::primitive::{Light, Primitive, Material, Intersection};


// TODO: clamp or bugs will be
pub fn tracer(camera: &Camera, scene: &Scene, buffer: &mut Vec<[u8; 4]>) {
    for y in 0..camera.width {
        for x in 0..camera.height {
            let ray = camera.generate(x, y);
            let color = trace(scene, ray);
            let idx = x + y * camera.height;
            // TODO we can use the saturated-add instruction here
            buffer[idx][0] = f32::min(255.0,(255.0 * color[0])) as u8;
            buffer[idx][1] = f32::min(255.0,(255.0 * color[1])) as u8;
            buffer[idx][2] = f32::min(255.0,(255.0 * color[2])) as u8;
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

            // TODO can be negative
            let l_dot_n = f32::max(0.0, light_direction.dot(intersection.normal));



            color +
                ((intersection.material.color * light.intensity * l_dot_n) /
                     (light_distance * light_distance))
        },
    )
}

fn trace(scene: &Scene, ray: Ray) -> Vector3<f32> {
    nearest_intersection(scene, ray)
        .map(|i| direct_illumination(scene, ray.direction, i))
        .unwrap_or(Vector3::new(0.0, 0.0, 0.0))
}
