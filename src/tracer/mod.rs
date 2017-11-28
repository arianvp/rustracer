pub mod camera;
pub mod ray;
use cgmath::{EuclideanSpace, InnerSpace};
use cgmath::{Vector3, Point3};
use std::f32;
use std::cmp::Ordering;


use self::ray::Ray;
use self::camera::Camera;

#[derive(Debug, Copy, Clone)]
struct Intersection {
    distance: f32, // here for convenience
    intersection: Point3<f32>,
    normal: Vector3<f32>,
    material: Material,
}

#[derive(Debug, Copy, Clone)]
struct Light {
   position: Point3<f32>,
   intensity: f32,
}

#[derive(Debug, Copy, Clone)]
struct Sphere {
    position: Point3<f32>,
    radius: f32,
    material: Material,
}

struct Plane {
    normal: Vector3<f32>,
    distance: f32,
    material: Material,
}


#[derive(Debug, Copy, Clone)]
struct Material {
    color: Vector3<f32>,
}

struct Scene {
    spheres: [Sphere;1],
    lights: [Light;1],
}

trait Primitive {
    fn intersect(&self, ray: &Ray) -> Option<Intersection>;
}

impl Plane {
    fn new(p1: Point3<f32>, p2: Point3<f32>, p3: Point3<f32>, material: Material)  -> Plane{
        let normal = (p3 - p1).cross(p2 - p1).normalize();
        Plane{
            normal,
            distance: -1.0*normal.dot(p1 - Point3::new(0.0,0.0,0.0)),
            material,
        }
    }
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        //let distance = self.position - ray.origin;
        let denom = ray.direction.dot(self.normal);
        if denom < f32::EPSILON {
            None
        } else {
            let t = (ray.origin.dot(self.normal) + self.distance) / ray.direction.dot(self.normal);
            let p = ray.origin + t * ray.direction;

            let distance = (p - ray.origin).magnitude();
            if t >= 0.0 {
                Some(Intersection{
                distance: distance ,
                intersection: p, 
                normal: self.normal,
                material: self.material,
                })
            } else {
                None
            }
        }
    }
}

impl Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
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

const SCENE: Scene = Scene {
    lights: [
        Light {
            intensity: 2.0,
            position: Point3 {
                x: 0.0,
                y: 2.0,
                z: 1.0,
            },
        },
    ],
    spheres: [
        Sphere {
            material: Material {
                color: Vector3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            position: Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            radius: 0.5,
        },
    ],
};
// TODO: clamp or bugs will be
pub fn tracer(camera: &Camera, buffer: &mut Vec<[u8; 4]>) {
    for y in 0..camera.width {
        for x in 0..camera.height {
            let ray = camera.generate(x, y);
            let color = trace(&SCENE, ray);
            let idx = x+y*camera.height;
            buffer[idx][0] = (255.0 * color[0]) as u8;
            buffer[idx][1] = (255.0 * color[1]) as u8;
            buffer[idx][2] = (255.0 * color[2]) as u8;
        }
    }

}

// what a beauty
fn nearest_intersection(scene: &Scene, ray: Ray) -> Option<Intersection> {
    scene
        .spheres
        .iter()
        .filter_map(|circle| circle.intersect(&ray))
        .min_by(|a,b| a.distance.partial_cmp(&b.distance).unwrap_or(Ordering::Equal))
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
            let l_dot_n = f32::max(0.0,light_direction.dot(intersection.normal));



            color + ((intersection.material.color  * light.intensity * l_dot_n) / (light_distance * light_distance))
        },
    )
}

fn trace(scene: &Scene, ray: Ray) -> Vector3<f32> {
    nearest_intersection(scene, ray)
        .map(|i| direct_illumination(scene, ray.direction, i))
        .unwrap_or(Vector3::new(0.0, 0.0, 0.0))
}
