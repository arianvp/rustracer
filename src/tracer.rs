use nalgebra::{Vector3, Point3};
use winit::VirtualKeyCode;
use std::collections::HashSet;
use bvh::aabb::{AABB, Bounded};
use bvh::ray::Intersection;
use bvh::ray::Ray;
use bvh::flat_bvh;
use bvh::bounding_hierarchy::BHShape;

#[derive(VulkanoShader)]
#[ty = "compute"]
#[path = "shaders/tracer.glsl.comp"]
#[allow(dead_code)]
struct Dummy;

fn aabb_to_aabb(aabb: AABB) -> ty::AABB {
    ty::AABB {
        _dummy0: [0; 4],
        min: [aabb.min.x, aabb.min.y, aabb.min.z],
        max: [aabb.max.x, aabb.max.y, aabb.max.z],
    }
}

pub fn node_to_node(node: flat_bvh::FlatNode) -> ty::Node {
    ty::Node {
        _dummy0: [0; 4],
        _dummy1: [0; 4],
        aabb: aabb_to_aabb(node.aabb),
        entry_index: node.entry_index,
        exit_index: node.exit_index,
        shape_index: node.shape_index,
    }
}

impl Bounded for ty::Triangle {
    fn aabb(&self) -> AABB {
        let p1 = Point3::new(self.p1[0], self.p1[1], self.p1[2]);
        let p2 = Point3::new(self.p2[0], self.p2[1], self.p2[2]);
        let p3 = Point3::new(self.p3[0], self.p3[1], self.p3[2]);
        AABB::empty().grow(&p1).grow(&p2).grow(&p3)
    }
}

impl BHShape for ty::Triangle {
    fn set_bh_node_index(&mut self, _index: usize) {}

    fn bh_node_index(&self) -> usize {
        0
    }

    fn intersect(&self, _ray: &Ray) -> Intersection {
        Intersection {
            distance: 0.0,
            u: 0.0,
            v: 0.0,
        }
    }
}

impl ty::Camera {
    pub fn new(origin: Vector3<f32>, target: Vector3<f32>, focal_distance: f32) -> ty::Camera {
        let mut camera = ty::Camera::_new(
            origin.into(),
            target.into(),
            [0.; 3],
            [0.; 3],
            [0.; 3],
            [0.; 3],
            [0.; 3],
            [0.; 3],
            focal_distance,
        );
        camera.update();
        camera
    }
    pub fn _new(
        origin: [f32; 3],
        target: [f32; 3],
        direction: [f32; 3],
        p1: [f32; 3],
        p2: [f32; 3],
        p3: [f32; 3],
        right: [f32; 3],
        up: [f32; 3],
        focal_distance: f32,
    ) -> ty::Camera {
        ty::Camera {
            origin,
            target,
            direction,
            p1,
            p2,
            p3,
            right,
            up,
            focal_distance,
            _dummy0: [0; 4],
            _dummy1: [0; 4],
            _dummy2: [0; 4],
            _dummy3: [0; 4],
            _dummy4: [0; 4],
            _dummy5: [0; 4],
            _dummy6: [0; 4],
        }
    }

    pub fn update(&mut self) {
        let target = Vector3::from(self.target);
        let origin = Vector3::from(self.origin);
        let direction = (target - origin).normalize();
        self.direction = direction.into();

        let unit_y = Vector3::new(0., 1., 0.);
        let right = unit_y.cross(&direction);
        self.right = right.into();
        let up = direction.cross(&right);
        println!("{:?}", up);
        println!("{:?}", right);
        self.up = up.into();

        let c = origin + self.focal_distance * direction;

        self.p1 = (c + (-0.5 * self.focal_distance * right) + (0.5 * self.focal_distance * up))
            .into();
        self.p2 = (c + (0.5 * self.focal_distance * right) + (0.5 * self.focal_distance * up))
            .into();
        self.p3 = (c + (-0.5 * self.focal_distance * right) + (-0.5 * self.focal_distance * up))
            .into();
    }

    pub fn handle_input(&mut self, keycodes: &HashSet<VirtualKeyCode>) {
        for keycode in keycodes {
            match *keycode {
                VirtualKeyCode::W => {
                    self.origin = (Vector3::from(self.origin) +
                        0.1 * Vector3::from(self.direction)).into();
                }
                VirtualKeyCode::A => {
                    self.origin = (Vector3::from(self.origin) + (-0.1 * Vector3::from(self.right)))
                        .into();
                    self.target = (Vector3::from(self.target) + (-0.1 * Vector3::from(self.right)))
                        .into();
                }
                VirtualKeyCode::S => {
                    self.origin = (Vector3::from(self.origin) +
                                       -0.1 * Vector3::from(self.direction)).into();
                }
                VirtualKeyCode::D => {
                    self.origin = (Vector3::from(self.origin) + (0.1 * Vector3::from(self.right)))
                        .into();
                    self.target = (Vector3::from(self.target) + (0.1 * Vector3::from(self.right)))
                        .into();
                }
                VirtualKeyCode::E => {
                    self.origin = (Vector3::from(self.origin) +
                                       10.0 * Vector3::from(self.direction)).into();
                    self.target = (Vector3::from(self.target) +
                                       10.0 * Vector3::from(self.direction)).into();
                }
                VirtualKeyCode::Q => {
                    self.origin = (Vector3::from(self.origin) +
                                       -10.0 * Vector3::from(self.direction)).into();
                    self.target = (Vector3::from(self.target) +
                                       -10.0 * Vector3::from(self.direction)).into();
                }
                VirtualKeyCode::R => {
                    self.origin = (Vector3::from(self.origin) + (0.1 * Vector3::from(self.up)))
                        .into();
                    self.target = (Vector3::from(self.target) + (0.1 * Vector3::from(self.up)))
                        .into();
                }
                VirtualKeyCode::F => {
                    self.origin = (Vector3::from(self.origin) + (-0.1 * Vector3::from(self.up)))
                        .into();
                    self.target = (Vector3::from(self.target) + (-0.1 * Vector3::from(self.up)))
                        .into();
                }
                VirtualKeyCode::Up => {
                    self.target = (Vector3::from(self.target) + (-0.1 * Vector3::from(self.up)))
                        .into();
                }
                VirtualKeyCode::Down => {
                    self.target = (Vector3::from(self.target) + (0.1 * Vector3::from(self.up)))
                        .into();
                }
                VirtualKeyCode::Left => {
                    self.target = (Vector3::from(self.target) + (-0.1 * Vector3::from(self.right)))
                        .into();
                }
                VirtualKeyCode::Right => {
                    self.target = (Vector3::from(self.target) + (0.1 * Vector3::from(self.right)))
                        .into();
                }
                _ => {}
            }
        }
        self.update();
    }
}

// extend:  find t for every ray
// shade: evaluate material at every t  (And do we direct_light_samplingd a shadow ray, and do we direct_light_samplingd to quit or not (russian))
//  -> shadow rays
//  -> path continuationrays
// connect   (will only trace shadow rays)     (is the only one that plot to the screen. Only shadow rays contribute)  (though not true for mirrors)
// jump back to to extend
// compaction can be done with atomic counter

