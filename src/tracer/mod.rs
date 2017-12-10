pub mod camera;
pub mod primitive;
pub mod ray;
pub mod scene;
pub mod mesh;

use cgmath::{InnerSpace, ElementWise, Array};
use cgmath::{Vector3, Point3};
use std::f32;
use std::mem;
use std::sync::Arc;
use morton;

use scoped_threadpool::Pool;


use half::f16;
use self::scene::Scene;
use self::ray::Ray;
use self::camera::Camera;
use self::primitive::{Light, Material};


struct Morton(*mut [f16; 4]);
unsafe impl Send for Morton {}
unsafe impl Sync for Morton {}

pub fn tracer(
    camera: &Camera,
    scene: &Scene,
    pool: &mut Pool,
    morton_lut: &Vec<(usize, usize)>,
    buffer: &mut Vec<[f16; 4]>,
) {
    let n = pool.thread_count() as usize;
    let mut_buffer = Arc::new(Morton(buffer.as_mut_ptr()));

    let all = morton_lut.chunks(128 * 128);


    // NOTE: this multi-threading is taken from my previous tracer on which I worked together on
    // with with Renier Maas, who did the course last year.


    pool.scoped(|scope| for chunk in all {
        let mut_buffer = mut_buffer.clone();
        scope.execute(move || for packet in chunk.chunks(4) {
            for &(x, y) in packet {
                let color = trace(scene, camera.generate(x, y), 5);

                let Morton(mut_buffer) = *mut_buffer;
                // TODO, it is hard to convince the borrow checker that accessing a buffer
                // in morton order is safe and doesn't cause any aliasing bugs
                unsafe {
                    for i in 0..3 {
                        (*mut_buffer.offset((x + y * camera.width) as isize))[i] =
                            f16::from_f32(color[i]);
                    }
                }
            }
        });
    });
}

pub fn brdf(intersection: Point3<f32>, normal: Vector3<f32>, light: &Light) -> f32 {
    let light_direction = light.position - intersection;
    let light_distance = light_direction.magnitude();
    let light_direction = light_direction.normalize();
    let l_dot_n = f32::max(0.0, light_direction.dot(normal));
    (light.intensity * l_dot_n) / (light_distance * light_distance)
}

fn direct_illumination(
    scene: &Scene,
    intersection: Point3<f32>,
    normal: Vector3<f32>,
    color: Vector3<f32>,
) -> Vector3<f32> {
    scene.lights.iter().fold(
        Vector3::new(0.0, 0.0, 0.0),
        |accum, light| {
            let mut to_mul = 0.0;
            let origin = intersection;
            let direction = (light.position - intersection).normalize();
            let ray = Ray {
                origin: origin + normal * BIAS,
                direction,
            };
            if normal.dot(direction) >= 0. && !scene.nearest_intersection(ray).is_some() {
                to_mul += brdf(intersection, normal, light) / 4.0
            }

            accum + (to_mul * color)
        },
    )
}

/// Calculates the refraction according Snellius Law Returns None when we're at the 'critical'
/// angle, that causes full internal reflection.
/// GLSL equiv: use https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/refract.xhtml
/// However in GLSL returns 0.0 in critical angle
fn refract(direction: Vector3<f32>, normal: Vector3<f32>, eta: f32) -> Option<Vector3<f32>> {
    let cos_phi_1 = -(normal.dot(direction));
    let k = 1.0 - eta * eta * (1.0 - cos_phi_1 * cos_phi_1);
    if k < 0.0 {
        None
    } else {
        let r = (eta * direction) + normal * (eta * cos_phi_1 - k.sqrt());
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


const BIAS: f32 = 0.001;



fn trace(scene: &Scene, ray: Ray, depth: u32) -> Vector3<f32> {
    if depth == 0 {
        return Vector3::new(0.0, 0.0, 0.0);
    }
    match scene.nearest_intersection(ray) {
        None => Vector3::new(0.1, 0.1, 1.0),
        Some(i) => {
            let biasn = BIAS * i.normal;
            match i.material {
                Material::Conductor { spec, color } => {
                    let s = if spec == 0.0 {
                        0.0
                    } else {
                        schlick(ray.direction, i.normal, spec)
                    };
                    let d = 1.0 - s;
                    let refraction = d *
                        direct_illumination(&scene, i.intersection, i.normal, color);
                    let r = reflect(ray.direction, i.normal);
                    // TODO break if not specular
                    let ray = Ray {
                        origin: i.intersection + biasn,
                        direction: r,
                    };
                    let reflection = if s == 0. {
                        Vector3::new(0., 0., 0.)
                    } else {
                        s * trace(scene, ray, depth - 1)
                    };
                    reflection + refraction
                }
                Material::Dielectric { absorbance, n1, n2 } => {
                    let outside = ray.direction.dot(i.normal) < 0.0;
                    let mut n1 = n1;
                    let mut n2 = n2;
                    if !outside {
                        mem::swap(&mut n1, &mut n2)
                    }
                    let norm_refrac = if outside { i.normal } else { -i.normal };
                    let bias_refrac = if outside { -biasn } else { biasn };
                    let n1n2 = n1 / n2;

                    let mut refl_amount =
                        schlick(ray.direction, norm_refrac, ((n2 - n1) / (n1 + n2)).powi(2));
                    let refr_amount = 1.0 - refl_amount;

                    let refr = if let Some(dir) = refract(ray.direction, norm_refrac, n1n2) {
                        let ray = Ray {
                            origin: i.intersection + bias_refrac,
                            direction: dir,
                        };
                        let distance = scene
                            .nearest_intersection(ray)
                            .map(|x| x.distance)
                            .unwrap_or(0.0);
                        let absorbance = absorbance * -distance;
                        let transparency = if outside {
                            Vector3::new(absorbance.x.exp(), absorbance.y.exp(), absorbance.z.exp())
                        } else {
                            Vector3::new(1.0, 1.0, 1.0)
                        };
                        let k = if refr_amount == 0. {
                            Vector3::new(0., 0., 0.)
                        } else {
                            trace(&scene, ray, depth - 1)
                        };
                        transparency.mul_element_wise(k)
                    } else {
                        refl_amount = 1.0;
                        Vector3::from_value(0.)
                    };

                    let refl = if refl_amount == 0. {
                        Vector3::new(0., 0., 0.)
                    } else {
                        trace(
                            &scene,
                            Ray {
                                origin: i.intersection + bias_refrac,
                                direction: reflect(ray.direction, i.normal),
                            },
                            depth - 1,
                        )
                    };
                    refl_amount * refl + refr_amount * refr

                }
            }
        }
    }
}
