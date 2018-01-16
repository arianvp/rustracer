use nalgebra::{Vector3, Point3};
use std::fmt::Debug;

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
};
//layout(std140, set = 0, binding = 2) buffer Spheres   { Sphere spheres[];   };
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

bool intersects_aabb(Ray ray, AABB aabb) {
  return false;
}

float intersects_triangle(Ray ray, Triangle triangle) {
  return 0.0;
}

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

    imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(vec3(ray.direction), 1.0));
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
}

// extend:  find t for every ray
// shade: evaluate material at every t  (And do we need a shadow ray, and do we need to quit or not (russian))
//  -> shadow rays
//  -> path continuationrays
// connect   (will only trace shadow rays)     (is the only one that plot to the screen. Only shadow rays contribute)  (though not true for mirrors)
// jump back to to extend
// compaction can be done with atomic counter
