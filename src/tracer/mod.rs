pub mod camera;
pub mod ray;

use self::camera::Camera;


fn tracer(camera: Camera, buffer: &mut [[f32; 3]]) {
    for y in 0..1024 {
        for x in 0..1024 {
            buffer[x+y*1024] = [1.0, 1.0, 1.0];
        }
    }
}

