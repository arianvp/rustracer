pub mod camera;
pub mod ray;
use cgmath::EuclideanSpace;
use cgmath::{Vector3, Point3};
use std::f32;

use self::ray::Ray;
use self::camera::Camera;


struct Intersection {
    distance: f32, // here for convenience
    intersection: Point3<f32>,
    normal: Vector3<f32>,
    material: Material,
}

pub struct Light {
    pub position: Point3<f32>,
    pub intensity: f32,
}

pub struct Sphere {
    pub position: Point3<f32>,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    fn intersect(&self, ray: Ray) -> Option<Intersection> {
        let distance = self.position - ray.origin;
        let tca = distance.dot(ray.direction);
        if tca < 0.0 {
            return None;
        }
        let d2 = distance.dot(distance) - tca * tca;
        if d2 > self.radius * self.radius {
            return None;
        }
        let thc = (self.radius * self.radius - d2).sqrt();
        let t0 = tca - thc;
        let t1 = tca + thc;
        if t0 >= 0.0 {
            let material = self.material;
            let distance = t0;
            let intersection = ray.origin + ray.direction * t0;
            let normal = intersection - self.position;
            Some(Intersection {
                material,
                normal,
                intersection,
                distance,
            })
        } else if t1 >= 0.0 {
            let material = self.material;
            let distance = t1;
            let intersection = ray.origin + ray.direction * t1;
            let normal = intersection - self.position;
            Some(Intersection {
                material,
                normal,
                intersection,
                distance,
            })
        } else {
            None
        }
    }
}

struct Scene {
    spheres: [Sphere],
    lights: [Light],
}
// currently only Phong shaded materials
struct Material {
    speculaty: f32,
    color: Vector3<f32>,
}


// TODO: clamp or bugs will be
pub fn tracer(camera: &Camera, buffer: &mut Vec<[u8; 4]>) {
    for y in 0..camera.width {
        for x in 0..camera.height {
            let ray = camera.generate(x, y);
            let color = cast(ray);
            /*buffer[idx][0] = (255.0 * color[0]) as u8;
            buffer[idx][1] = (255.0 * color[1]) as u8;
            buffer[idx][2] = (255.0 * color[2]) as u8;*/
        }
    }

}

// what a beauty
fn nearest_intersection(scene: &Scene, ray: Ray) -> Option<Intersection> {
    scene
        .circles
        .iter()
        .filter_map(|circle| circle.intersect(ray))
        .max_by_key(|intersection| intersection.distance)
        .next()
}

fn direct_illumination(
    scence: &Scene,
    ray_direction: Vector3<f32>,
    position: Vector3<f32>,
    normal: Vector3<f32>,
    material: Material,
) -> Vector3<f32> {
    scene.lights.iter().fold(
        Vector3::new(0, 0, 0),
        |specular_color, light| {
            let light_direction = light.position - position;
            let light_distance2 = light_direction.dot(light_direction);
            let light_direction = light_direction.normalize();
            let l_dot_n = max(0.0, light_direction.dot(nornal));
            let light_amount = light.intensity * l_dot_n;
            let reflection_direction = -light_direction.reflect(normal);
            f32::max(0.0, -reflection_direction.dot(ray_direction)).pow(material.specular_exp) *
                light.intensity;

        },
    );
}

fn trace(scence: &Scene, ray: Ray) -> Vector3<f32> {
    let intersection = nearest_intersection(scene, ray);
}
