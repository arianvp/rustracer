use nalgebra::{Vector3, Point3};
use std::fmt::Debug;
use winit::VirtualKeyCode;

#[derive(VulkanoShader)]
#[ty = "compute"]
#[src = "
#version 450

#define MAX_VALUE 5000

struct AABB {
  vec3 min;
  vec3 max;
};


// A Rope BVH
struct Node {
  AABB aabb;
  uint entry_index;
  uint exit_index;
  uint shape_index;
};


struct Triangle {
  vec3 p1;
  vec3 p2;
  vec3 p3;
};

struct Camera {
  vec3 origin;
  vec3 target;
  vec3 direction;
  vec3 p1;
  vec3 p2;
  vec3 p3;
  vec3 right;
  vec3 up;
  float focal_distance;
};

struct Sphere {
  vec3 position;
  float radius;
};

struct Ray {
  vec3 origin;
  vec3 direction;
};


layout(local_size_x = 16, local_size_y = 16) in;
layout(        set = 0, binding = 0, rgba8) uniform writeonly image2D img;
layout(std140, set = 0, binding = 1       ) uniform readonly Input {
  Camera camera;
  uint num_spheres;
};
layout(std140, set = 0, binding = 2) buffer Spheres   { Sphere spheres[];   };
//layout(std140, set = 0, binding = 3) buffer Positions { vec3   positions[]; };
//layout(std140, set = 0, binding = 4) buffer Indices   { uvec3  indices[];   };
//layout(std140, set = 0, binding = 5) buffer BVH       { Node   nodes[];     };
//layout(std140, set = 0, binding = 6) buffer Length    { uint   node_length; };



/*Triangle get_triangle(uint idx) {
  Triangle triangle;
  triangle.p1 = positions[indices[idx].x];
  triangle.p2 = positions[indices[idx].y];
  triangle.p3 = positions[indices[idx].z];
  return triangle;
}*/

/*float intersect_scene(Ray ray) {
  uint idx = 0;
  uint max_length = node_length;
  float best_time = 1.0 / 0.0;
  while (idx < max_length) {
    Node node = nodes[idx];
    if (node.entry_index == MAX_VALUE) { // leaf node
      Triangle triangle = get_triangle(node.shape_index);
      idx = node.exit_index;
      if (intersects_aabb(ray, node.aabb)) {
        float time = intersects_triangle(ray, triangle);
        if (time < best_time) {
          best_time = time;
        }
      }
    } else if (intersects_aabb(ray, node.aabb)) { // intersects internal node
      idx = node.entry_index;
    } else {
      idx = node.exit_index;
    }
  }
  return best_time;
}*/


bool intersects_aabb(Ray ray, AABB aabb) {
  return false;
}

float intersects_sphere(Ray ray, Sphere sphere) {
  vec3 distance = sphere.position - ray.origin;
  float tca = dot(distance, ray.direction);
  if (tca < 0.0) {
    return 1.0 / 0.0;
  }
  float  d2 = dot(distance, distance) - tca * tca;
  float r2 = sphere.radius * sphere.radius;
  if (d2 > r2) {
    return 1.0 / 0.0;
  }

  float thc = sqrt(r2 - d2);
  float t0 = tca - thc;
  float t1 = tca + thc;
  if (t0 > t1) {
    float temp = t0;
    t0 = t1;
    t1 = temp;
  }
  if (t0 < 0.0) {
    t0 = t1;
    if (t0 < 0.0) {
      return 1.0 / 0.0;
    }
  }
  return t0;
}

uint wang_hash(uint seed) {
  seed = (seed ^ 61) ^ (seed >> 16);
  seed *= 9;
  seed = seed ^ (seed >> 4);
  seed *= 0x27d4eb2d;
  seed = seed ^ (seed >> 15);
  return seed;
}

uvec4 next(uvec4 ctx) {
  uint t = ctx.x ^ (ctx.x << 11);
  ctx = ctx.yzww;
  ctx.w = ctx.w & (ctx.w >> 19) ^ (t ^ (t >> 8));
  return ctx;
}

vec3 sample_ray(Ray ray) {
  vec3 transmission = vec3(1.0, 1.0, 1.0);
  vec3 emission = vec3(0.0, 0.0, 0.0);
  return emission;
}


Ray generate_ray(vec2 uv) {
  vec3 t = camera.p1 + uv.x * (camera.p2 - camera.p1) + uv.y * (camera.p3 - camera.p1);
  vec3 origin = camera.origin;
  vec3 direction = normalize(t - origin);
  Ray ray = {origin, direction};
  return ray;
}

void main() {
    uint seed = wang_hash(wang_hash(gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * imageSize(img).x));
    float val = seed * (1.0 / 4294967269.0);

    vec2 uv = vec2(gl_GlobalInvocationID.xy) / imageSize(img);
    Ray ray = generate_ray(uv);

    uint best_i;
    uint i;
    float t  = 1.0 / 0.0;
    for (i = 0; i < num_spheres; i++) {
      float t_new = intersects_sphere(ray, spheres[i]);
      if (t_new < t) { t = t_new; best_i = i; }
    }

    if (t < 1.0 / 0.0) {
      vec3 intersection = ray.origin + ray.direction * t;
      vec3 normal = normalize(intersection - spheres[best_i].position);
      
      imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(normal, 1.0));
    }

}
"]
#[allow(dead_code)]
struct Dummy;


impl ty::Camera {
    pub fn new(origin: Vector3<f32>, target: Vector3<f32>, focal_distance: f32) -> ty::Camera {
        let mut camera = ty::Camera::_new(origin.into(), target.into(), [0.;3], [0.;3], [0.;3], [0.;3],[0.;3],[0.;3], focal_distance);
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
            origin, target, direction, p1, p2, p3, right, up, focal_distance,
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
        self.up = up.into();

        let c = origin + self.focal_distance * direction;

        self.p1 = (c + (-0.5 * self.focal_distance * right) + (0.5 * self.focal_distance * up)).into();
        self.p2 = (c + (0.5 * self.focal_distance * right) + (0.5 * self.focal_distance * up)).into();
        self.p3 = (c + (-0.5 * self.focal_distance * right) + (-0.5 * self.focal_distance * up)).into();
    }

   pub fn handle_input(&mut self, keycode: VirtualKeyCode) {
        match keycode {
            VirtualKeyCode::W => {
                self.origin = (Vector3::from(self.origin) + 0.1 * Vector3::from(self.direction)).into();
            },
            VirtualKeyCode::A => {
                self.origin = (Vector3::from(self.origin) + (-0.1 * Vector3::from(self.right))).into();
                self.target = (Vector3::from(self.target) + (-0.1 * Vector3::from(self.right))).into();
            },
            VirtualKeyCode::S => {
                self.origin = (Vector3::from(self.origin) + -0.1 * Vector3::from(self.direction)).into();
            },
            VirtualKeyCode::D => {
                self.origin = (Vector3::from(self.origin) + (0.1 * Vector3::from(self.right))).into();
                self.target = (Vector3::from(self.target) + (0.1 * Vector3::from(self.right))).into();
            },
            VirtualKeyCode::E => {
                self.origin = (Vector3::from(self.origin) + 10.0 * Vector3::from(self.direction)).into();
                self.target = (Vector3::from(self.target) + 10.0 * Vector3::from(self.direction)).into();
            },
            VirtualKeyCode::Q => {
                self.origin = (Vector3::from(self.origin) + -10.0 * Vector3::from(self.direction)).into();
                self.target = (Vector3::from(self.target) + -10.0 * Vector3::from(self.direction)).into();
            },
            VirtualKeyCode::R => {
                self.origin = (Vector3::from(self.origin) + (0.1 * Vector3::from(self.up))).into();
                self.target = (Vector3::from(self.target) + (0.1 * Vector3::from(self.up))).into();
            },
            VirtualKeyCode::F => {
                self.origin = (Vector3::from(self.origin) + (-0.1 * Vector3::from(self.up))).into();
                self.target = (Vector3::from(self.target) + (-0.1 * Vector3::from(self.up))).into();
            },
            VirtualKeyCode::Up => {
                self.target = (Vector3::from(self.target) + (-0.1 * Vector3::from(self.up))).into();
            },
            VirtualKeyCode::Down => {
                self.target = (Vector3::from(self.target) + (0.1 * Vector3::from(self.up))).into();
            },
            VirtualKeyCode::Left => {
                self.target = (Vector3::from(self.target) + (-0.1 * Vector3::from(self.right))).into();
            },
            VirtualKeyCode::Right => {
                self.target = (Vector3::from(self.target) + (0.1 * Vector3::from(self.right))).into();
            },
            _ => {},
        }
        self.update();
    }
}

// extend:  find t for every ray
// shade: evaluate material at every t  (And do we need a shadow ray, and do we need to quit or not (russian))
//  -> shadow rays
//  -> path continuationrays
// connect   (will only trace shadow rays)     (is the only one that plot to the screen. Only shadow rays contribute)  (though not true for mirrors)
// jump back to to extend
// compaction can be done with atomic counter
