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
pub fn brdf(intersection: Point3<f32>, normal: Vector3<f32>, light: &Light) -> f32 {
    let light_direction = light.position - intersection;
    let light_distance = light_direction.magnitude();
    let light_direction = light_direction.normalize();
    let l_dot_n = f32::max(0.0, light_direction.dot(normal));
    (light.intensity * l_dot_n) / (light_distance * light_distance)
}

fn direct_illumination(scene: &Scene, intersection: Point3<f32>, normal: Vector3<f32>, color: Vector3<f32>) -> Vector3<f32> {
    scene.lights.iter().fold(
        Vector3::new(0.0, 0.0, 0.0),
        |accum, light| {
            let origin = intersection;
            let direction = (light.position - intersection).normalize();
            let ray = Ray { origin: origin + normal * bias, direction };
            let to_mul = if nearest_intersection(scene, ray).is_some() {
                Vector3::new(0.0, 0.0, 0.0)
            } else {
                brdf(intersection, normal, light) * color
            };

            accum + to_mul
        },
    )
}

/// Calculates the refraction according Snellius Law Returns None when we're at the 'critical'
/// angle, that causes full internal reflection.
/// GLSL equiv: use https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/refract.xhtml
/// However in GLSL returns 0.0 in critical angle
fn refract(
    direction: Vector3<f32>,
    normal: Vector3<f32>,
    eta: f32,
) -> Option<Vector3<f32>> {
    let cos_phi_1 = normal.dot(direction);
    let k = 1.0 - eta * eta * (1.0 - cos_phi_1 * cos_phi_1);
    if k < 0.0 {
        None
    } else {
        let r = (eta * direction) - (eta * cos_phi_1 + k.sqrt()) * normal;
        Some(r)
    }
}

/// Calculates the reflection vector
fn reflect(direction: Vector3<f32>, normal: Vector3<f32>) -> Vector3<f32> {
    (direction - 2. * direction.dot(normal) * normal)
}

/// Approximates the Frensel equations for a dielectric-conductor interface
/// Borrowed from : https://seblagarde.wordpress.com/2013/04/29/memo-on-fresnel-equations/
///
///
/// Approximates the Frensel equations for two dielectrics interfacing
fn schlick(direction: Vector3<f32>, normal: Vector3<f32>, r0: f32) -> f32 {
    r0 + (1.0 - r0) * (1.0 - -direction.dot(normal)).powi(5)
}

const bias: f32 = 0.001;

fn trace(scene: &Scene, ray: Ray, depth: u32) -> Vector3<f32> {
    let mut ray = ray.clone();
    let mut a = 1.0;
    let mut b = 1.0;
    let mut accum = Vector3::new(0.0, 0.0, 0.0);
    for j in 0..depth {
        match nearest_intersection(scene, ray) {
            None => {
                break;
            }
            Some(i) => {
                let outside = ray.direction.dot(i.normal) < 0.0;
                let biasn = bias * i.normal;
                match i.material {
                    Material::Conductor{spec, color} => {
                        let r = reflect(ray.direction, i.normal);
                        let s = spec; // schlick(ray.direction, i.normal, spec);
                        let d = 1.0 - s;
                        accum += a * d * direct_illumination(&scene, i.intersection, i.normal, color);
                        a *= s;
                        let r = reflect(ray.direction, i.normal);
                        // TODO break if not specular
                        ray = Ray {
                            origin: i.intersection + if outside { biasn } else { -biasn },
                            direction: r,
                        }
                    },
                    Material::Dielectric{color, n1, n2} => {
                        let mut reflect_ = schlick(ray.direction, i.normal, ((n1-n2) / (n1+n2)).powi(2));
                        let refract_ = 1.0 - reflect_;

                        let r = reflect(ray.direction, i.normal);
                        accum += b * reflect_ * trace(&scene, Ray{origin: i.intersection + if outside {biasn} else {-biasn}, direction:r}, depth - j - 1);

                        // Now we need to send two rays. But my framework does not support this. So
                        // recursion
                        if let Some(r) = refract(ray.direction, i.normal, if outside { 1.0 / 1.125} else { 1.125 / 1.0 }) {
                            b *= refract_;
                            ray = Ray{origin: i.intersection + if outside {-biasn} else {biasn}, direction: r}
                        } else {
                            reflect_ = 1.0;
                        }

                    },
                }
                // TODO reword specularity in terms of the Frensel equations, also for Lambertians
            }
        }
    }
    accum
}
