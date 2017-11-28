use cgmath::{Vector3, Point3, Array};
use cgmath::InnerSpace;
use cgmath::ElementWise;
use winit::VirtualKeyCode;
use super::ray::Ray;
use std::f32;


#[derive(Debug)]
pub struct Camera {
    origin: Point3<f32>,
    target: Point3<f32>,
    focal_distance: f32,
    direction: Vector3<f32>,

    // screen plane
    p1: Point3<f32>,
    p2: Point3<f32>,
    p3: Point3<f32>,

    up: Vector3<f32>,
    right: Vector3<f32>,
    pub width: usize,
    pub height: usize,
    lens_size: f32,

    depth: u32,
}


impl Camera {
    pub fn new(width: usize, height: usize) -> Camera {
        let mut camera = Camera {
            width: width,
            height: height,
            depth: 512,
            lens_size: 0.10,
            origin: Point3::new(-1.6, 0.0, -1.3), //normal
            target: Point3::new(0.7, 0.0, 0.6),
            direction: Vector3::new(0.0, 0.0, 0.0),
            focal_distance: 0.0,
            p1: Point3::new(0.0, 0.0, 0.0),
            p2: Point3::new(0.0, 0.0, 0.0),
            p3: Point3::new(0.0, 0.0, 0.0),
            right: Vector3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 0.0, 0.0),
        };
        camera.update();
        camera
    }


    pub fn update(&mut self) {
        self.direction = (self.target - self.origin).normalize();
        let unit_y = Vector3::new(0.0, 1.0, 0.0);
        self.right = unit_y.cross(self.direction);
        self.up = self.direction.cross(self.right);


        let aspect_ratio = (self.width as f32) / (self.height as f32);

        self.focal_distance = 20.0;

        let c = self.origin + self.focal_distance * self.direction;

        self.p1 = c + (-0.5 * self.focal_distance * aspect_ratio * self.right) +
            (0.5 * self.focal_distance * self.up);
        self.p2 = c + (0.5 * self.focal_distance * aspect_ratio * self.right) +
            (0.5 * self.focal_distance * self.up);
        self.p3 = c + (-0.5 * self.focal_distance * aspect_ratio * self.right) +
            (-0.5 * self.focal_distance * self.up);

    }

    pub fn handle_input(&mut self, keycode: VirtualKeyCode) {
        match keycode {
            VirtualKeyCode::W => {
                self.origin += (0.1 * self.direction);
            },
            VirtualKeyCode::A => {
                self.origin = self.origin + (-0.1 * self.right);
                self.target = self.target + (-0.1 * self.right);
            },
            VirtualKeyCode::S => {
                self.origin += (-0.1 * self.direction);
            },
            VirtualKeyCode::D => {
                self.origin = self.origin + (0.1 * self.right);
                self.target = self.target + (0.1 * self.right);
            },
            VirtualKeyCode::Up => {
                self.target = self.target + (-0.1 * self.up);
            },
            VirtualKeyCode::Down => {
                self.target = self.target + (0.1 * self.up);
            },
            VirtualKeyCode::Left => {
                self.target = self.target + (-0.1 * self.right);
            },
            VirtualKeyCode::Right => {
                self.target = self.target + (0.1 * self.right);
            },
            _ => {},
        }
        self.update();
    }


    /// generates a nice Ray (TODO better integer type)
    pub fn generate(&self, x: usize, y: usize) -> Ray {
        // NOTE: we do not have to keep track of a
        // pool of random number generators, each
        // thread in rust has its own random
        // number generator by default :)

        // calculate sub-pixel ray target position on screen plane
        // TODO simd this
        let u = ((x as f32)) / (self.width as f32);
        let v = ((y as f32)) / (self.height as f32);
        let target = self.p1 + u * (self.p2 - self.p1) + v * (self.p3 - self.p1);
        let origin = self.origin + self.lens_size * (self.right + self.up);
        let direction = (target - origin).normalize();

        Ray{origin, direction}
    }
}
