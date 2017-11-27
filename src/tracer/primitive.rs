
use cgmath::{Point3, InnerSpace, Vector3};
use tracer::ray::Ray;

pub struct Sphere {
    pub position: Point3<f32>,
    pub radius: f32,
}

impl Sphere {
    fn intersect(&self, ray: &mut Ray) -> bool {
        let distance = self.position - ray.origin;
        let tca = distance.dot(ray.direction);

        if tca < 0.0 {
            return false;
        }

        let d2 = distance.dot(distance) - tca * tca;

        if d2 > self.radius * self.radius {
            return false;
        }

        let thc = (self.radius * self.radius - d2).sqrt();
        let t0 = tca - thc;
        let t1 = tca + thc;

        if t0 >= 0.0 {
            if t0 >= ray.distance {
                false
            } else {
                ray.distance = t0;
                true
            }
        } else if t1 >= 0.0 {
            if t1 >= ray.distance {
                false
            } else {
                ray.distance = t1;
                true
            }
        } else {
            false
        }
    }
}
