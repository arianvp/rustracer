use nalgebra::{Vector3, Point3};
use std::fmt::Debug;
use winit::VirtualKeyCode;
use std::collections::HashSet;

#[derive(VulkanoShader)]
#[ty = "compute"]
#[src = "
#version 450

#define MAX_VALUE (5000)
#define PI (3.1415926535359)
#define INV_PI (1.0 / PI)
#define EPSILON (0.0001)

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

// simple. no frensel term yet
struct Material {
  // if emissive, then refl is the amount of light
  uint emissive;
  float refl;
  vec3  diffuse;
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
  Material material;
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
  int frame_num;
};
layout(std140, set = 0, binding = 2) buffer Spheres   { Sphere spheres[];   };
layout(        set = 0, binding = 3) buffer Accum     { vec3 accum[];       };

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

float next_float(inout uvec4 rng) {
  uint t = rng.x ^ (rng.x << 11);
  rng = rng.yzww;
  rng.w = rng.w & (rng.w >> 19) ^ (t ^ (t >> 8));
  return float(rng.w) * (1.0 / 4294967296.0);
}

float hash1(inout float seed) {
    return fract(sin(seed += 0.1)*43758.5453123);
}

vec2 hash2(inout float seed) {
    return fract(sin(vec2(seed+=0.1,seed+=0.1))*vec2(43758.5453123,22578.1459123));
}

vec3 hash3(inout float seed) {
    return fract(sin(vec3(seed+=0.1,seed+=0.1,seed+=0.1))*vec3(43758.5453123,22578.1459123,19642.3490423));
}


Ray generate_ray(vec2 uv) {
  vec3 t = camera.p1 + uv.x * (camera.p2 - camera.p1) + uv.y * (camera.p3 - camera.p1);
  vec3 origin = camera.origin;
  vec3 direction = normalize(t - origin);
  Ray ray = {origin, direction};
  return ray;
}


vec3 cosWeightedRandomHemisphereDirection( const vec3 n, inout float seed ) {
  	vec2 r = hash2(seed);
    
	vec3  uu = normalize( cross( n, vec3(0.0,1.0,1.0) ) );
	vec3  vv = cross( uu, n );
	
	float ra = sqrt(r.y);
	float rx = ra*cos(2*PI*r.x); 
	float ry = ra*sin(2*PI*r.x);
	float rz = sqrt( 1.0-r.y );
	vec3  rr = vec3( rx*uu + ry*vv + rz*n );
    
    return normalize( rr );
}

vec3 trace(Ray ray, inout float seed) {
    vec3 emit = vec3(0.0);
    vec3 trans = vec3(1.0);

    for (int j = 0; j < 8; j++) {
      int best_j;
      float t  = 1.0 / 0.0;
      for (int j = 0; j < num_spheres; j++) {
        float t_new = intersects_sphere(ray, spheres[j]);
        if (t_new < t) { t = t_new; best_j = j; }
      }
      if (t >= 1.0 / 0.0) {
        emit = vec3(0.0);
        break;
      }


      Material material = spheres[best_j].material;

      if (material.emissive == 1) {
        emit += trans * material.diffuse;
        break;
      }

      vec3 intersection = ray.origin + ray.direction * t;
      vec3 normal = normalize(intersection - spheres[best_j].position);

      ray.origin = intersection + normal * EPSILON;
      ray.direction = cosWeightedRandomHemisphereDirection(normal, seed);
      trans *= material.diffuse;

    }

    return emit;
}

/*vec3 sample_ray(Ray ray, uvec4 rng) {
  vec3 s = vec3(0.0);
  vec3 mask = vec3(1.0);
  uint i;
  for (i = 0; i < 12; i++) {

    uint best_j;
    uint j;
    float t  = 1.0 / 0.0;
    for (j = 0; j < num_spheres; j++) {
      float t_new = intersects_sphere(ray, spheres[j]);
      if (t_new < t) { t = t_new; best_j = j; }
    }
    if (t >= 1.0 / 0.0) {
      s = vec3(0.0);
      break;
    }; 
    Material material = spheres[best_j].material;

    vec3 intersection = ray.origin + ray.direction * t;
    vec3 normal = normalize(intersection - spheres[best_j].position);

    uvec4 rng1 = next(rng);
    uvec4 rng2 = next(rng1);
    rng  = rng2;

    float rand1 = float(rng1.w) * (1.0 / 4294967296.0);
    float rand2 = float(rng2.w) * (1.0 / 4294967296.0);
    float rand2s = sqrt(rand2);


    vec3 normal_facing = dot(normal, ray.direction) < 0.0 ? normal : normal * (-1.0f);
    vec3 w = normal_facing;
    vec3 axis = (abs(w.x) > 0.1) ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0);
    vec3 u = normalize(cross(axis,w));
    vec3 v = cross(w,u);

    vec3 newdir = normalize(u * cos(rand1)*rand2s + v*sin(rand1)*rand2s + w*sqrt(1.0-rand2));
    ray.origin = intersection + normal_facing * EPSILON;
    ray.direction = newdir;


    s += mask * material.diffuse * float(material.emissive + 1);
    mask *= material.diffuse;
    mask *= dot(newdir, normal_facing);

  }
  return s;
}*/

void main() {

    //imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(vec3(0.0), 1.0)); 
    uint idx = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * imageSize(img).x;

    if (frame_num == 1) {
        accum[idx] = vec3(0.0);
        imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(vec3(0.0), 1.0)); 
    }

    float seed = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * 3.43121412313 + fract(1.12345314312*float(frame_num));


    vec2 uv = vec2(gl_GlobalInvocationID.xy) / imageSize(img);

    Ray ray = generate_ray(uv);
    vec3 color = trace(ray, seed);

    accum[idx] += color;
    vec3 outCol = accum[idx] / float(frame_num);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(outCol, 1.0));


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

   pub fn handle_input(&mut self, keycodes: &HashSet<VirtualKeyCode>) {
       for keycode in keycodes {
            match *keycode {
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
