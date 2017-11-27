pub mod camera;
pub mod ray;
pub mod primitive;
use self::ray::Ray;
use cgmath::EuclideanSpace;

use self::camera::Camera;


pub fn tracer(camera: &Camera, buffer: &mut Vec<[u8; 4]>) {
    for y in 0..camera.width {
        for x in 0..camera.height {
            let ray = 255.0*camera.generate(x,y).direction;
            let idx = x+y*camera.width;
            buffer[idx][0] = ray[0] as u8;
            buffer[idx][1] = ray[1] as u8;
            buffer[idx][2] = ray[2] as u8;
        }
    }

}

fn trace(ray: Ray) {
}

