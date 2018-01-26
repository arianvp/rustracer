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

// wide hash + deep lcg from 
// http://www.reedbeta.com/blog/quick-and-easy-gpu-random-numbers-in-d3d11/
uint wang_hash(uint seed) {
  seed = (seed ^ 61) ^ (seed >> 16);
  seed *= 9;
  seed = seed ^ (seed >> 4);
  seed *= 0x27d4eb2d;
  seed = seed ^ (seed >> 15);
  return seed;
}

float next_float_lcg(inout uint state) {
  state = 1664525 * state + 1013904223;
  return state * (1.0 / 4294967296.0);
}

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

// simple. no frensel term yet
struct Material {
  // if emissive, then refl is the amount of light
  uint emissive;
  float refl;
  vec3  diffuse;
};

struct Plane {
    vec3 normal;
    float d;
    Material material;
};

struct Triangle {
  vec3 p1;
  vec3 p2;
  vec3 p3;
  vec3 normal;
  Material material;
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
  vec3 inv_direction;
};


layout(local_size_x = 16, local_size_y = 16) in;
layout(        set = 0, binding = 0, rgba8) uniform writeonly image2D img;
layout(std140, set = 0, binding = 1       ) uniform readonly Input {
  Camera camera;
  uint num_spheres;
  uint num_planes;
  uint num_triangles;
  uint frame_num;
  uint node_length;
  Triangle light;
  int debug;
};
layout(std140, set = 0, binding = 2) buffer Spheres   { Sphere spheres[];   };
layout(std140, set = 0, binding = 3) buffer Planes    { Plane  planes[];    };
layout(std140, set = 0, binding = 4) buffer Triangles { Triangle triangles[]; };
layout(        set = 0, binding = 5) buffer Accum     { vec3   accum[];     };

layout(std140, set = 0, binding = 6) buffer BVH       { Node   nodes[];     };

bool intersects_aabb(Ray ray, AABB aabb) {
  float tx1 = (aabb.min.x - ray.origin.x) * ray.inv_direction.x;
  float tx2 = (aabb.max.x - ray.origin.x) * ray.inv_direction.x;
  
  float tmin = min(tx1, tx2);
  float tmax = max(tx1, tx2);
  
  float ty1 = (aabb.min.y - ray.origin.y) * ray.inv_direction.y;
  float ty2 = (aabb.max.y - ray.origin.y) * ray.inv_direction.y;
  
  tmin = max(tmin, min(ty1, ty2));
  tmax = min(tmax, max(ty1, ty2));
  
  float tz1 = (aabb.min.z - ray.origin.z) * ray.inv_direction.z;
  float tz2 = (aabb.max.z - ray.origin.z) * ray.inv_direction.z;
 
  tmin = max(tmin, min(tz1, tz2));
  tmax = min(tmax, max(tz1, tz2));
  
  return tmax > tmin && tmax > 0.;
}


float intersects_plane(Ray ray, Plane plane) {
  return (-plane.d - dot(plane.normal, ray.origin)) / dot(plane.normal, ray.direction);
}

vec3 random_point_on_triangle(const Triangle triangle, inout uint seed) {
  float u = next_float_lcg(seed);
  float v = next_float_lcg(seed);
  if (u + v >= 1.) {
    u = (1 - u);
    v = (1 - v);
  }
  vec3 e1 = triangle.p2 - triangle.p1;
  vec3 e2 = triangle.p3 - triangle.p1;
  return triangle.p1 + u * e1 + v * e2;
}


float triangle_area(const Triangle triangle) {
  vec3 e1 = triangle.p2 - triangle.p1;
  vec3 e2 = triangle.p3 - triangle.p1;
  float l1 = length(e1);
  float l2 = length(e2);
  return 0.5 * (1.0 - dot(e1/l1, e2/l2)) * l1 * l2;
}

float intersects_triangle(Ray ray, Triangle triangle) {
    vec3 e1 = triangle.p2 - triangle.p1;
    vec3 e2 = triangle.p3 - triangle.p1;
    vec3 p = cross(ray.direction, e2);
    float det = dot(e1, p);

    if (det < EPSILON) {
        return 1.0e34;
    }

    float inv_det = 1.0 / det;
    vec3 tk = ray.origin - triangle.p1;
    float u = dot(tk,p) * inv_det;
    if (u < 0.0 || u > 1.0) {
        return 1.0e34;
    }
    vec3 q = cross(tk,e1);
    float v = dot(ray.direction, q) * inv_det;
    if (v < 0.0 || u + v > 1.0) {
        return 1.0e34;
    }
    float t = dot(e2, q) * inv_det;
    return t; 
    //} else {
    // return 1.0e34;
   // }
}


float intersects_sphere(Ray ray, Sphere sphere) {
  vec3 distance = sphere.position - ray.origin;
  float tca = dot(distance, ray.direction);
  if (tca < 0.0) {
    return 1.0e34;
  }
  float  d2 = dot(distance, distance) - tca * tca;
  float r2 = sphere.radius * sphere.radius;
  if (d2 > r2) {
    return 1.0e34;
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
      return 1.0e34;
    }
  }
  return t0;
}


Ray generate_ray(vec2 uv) {
  vec3 t = camera.p1 + uv.x * (camera.p2 - camera.p1) + uv.y * (camera.p3 - camera.p1);
  vec3 origin = camera.origin;
  vec3 direction = normalize(t - origin);
  Ray ray = {origin, direction, vec3(1.0)/direction};
  return ray;
}


vec3 diffuse_reflection(inout uint seed) {
  // based on SmallVCM / GIC
  float r1 = next_float_lcg(seed);
  float r2 = next_float_lcg(seed);
  float term1 = 2 * PI * r1;
  float term2 = 2 * sqrt( r2 * (1 - r2) );
  vec3 R = vec3( cos( term1 ) * term2, sin( term1 ) * term2, 1 - 2 * r2 );
  if (R.z < 0) R.z = -R.z;
  return R;
}

vec3 local_to_world(const vec3 V, const vec3 N ) {
  // based on SmallVCM
  vec3 tmp = (abs( N.x ) > 0.99f) ? vec3( 0, 1, 0 ) : vec3( 1, 0, 0 );
  vec3 B = normalize( cross( N, tmp ) );
  vec3 T = cross( B, N );
  return V.x * T + V.y * B + V.z * N;
}

vec3 world_to_local(const vec3 V, const vec3 N ) {
  vec3 tmp = (abs( N.x ) > 0.99f) ? vec3( 0, 1, 0 ) : vec3( 1, 0, 0 );
  vec3 B = normalize( cross( N, tmp ) );
  vec3 T = cross( B, N );
  return vec3( dot( V, T ), dot( V, B ), dot( V, N ) );
}


vec3 diffuse_reflection_cos(inout uint seed)
{
  // based on SmallVCM
   float r0 = next_float_lcg(seed);
   float r1 = next_float_lcg(seed);
  float term1 = 2 * PI * r0;
  float term2 = sqrt( 1 - r1 );
  return vec3( cos( term1 ) * term2, sin( term1 ) * term2, sqrt( r1 ) );
}


void intersect_bvh(Ray ray, inout int best_j, inout float best_t, inout int typ, inout float bvh) {
    uint index = 0;
    while (index < node_length) {
        Node node = nodes[index];
        if (node.entry_index == 4294967295) {
            Triangle triangle = triangles[node.shape_index];
            if (intersects_aabb(ray, node.aabb)) {
               float t = intersects_triangle(ray, triangle);
               if (t < best_t) {
                typ = 1;
                best_t = t;
                best_j = int(node.shape_index);
               }
            }
            index = node.exit_index;
        } else if (intersects_aabb(ray, node.aabb)) {
            index = node.entry_index;
            bvh += 0.001;
        } else {
            index = node.exit_index;
        }
    }
}

float intersect_shadow(const Ray ray, float t) {
    for (int j = 0; j < num_spheres; j++) {
      float t_new = intersects_sphere(ray, spheres[j]);
      if (t_new < t)  t = t_new;
    }
    int best_j;
    int typ;
    float bvh;
    intersect_bvh(ray, best_j, t, typ, bvh);
    return t;
}

void intersect(const Ray ray, inout int typ, inout int best_j, inout float t, inout float bvh) {
    for (int j = 0; j < num_planes; j++) {
      float t_new = intersects_plane(ray, planes[j]);
      if (t_new < EPSILON) {
        t_new = 1.0e34;
      }
      if (t_new < t) { t = t_new; best_j = j; typ = 0; }
    }

    float t_new = intersects_triangle(ray, light);
    if (t_new < t) { t = t_new; best_j = -1; typ = -1; }

    /*for (int j = 0; j < num_triangles; j++) {
      float t_new = intersects_triangle(ray, triangles[j]);
      if (t_new < t) { t = t_new; best_j = j; typ = 1; }
    }*/
   
    intersect_bvh(ray, best_j, t, typ, bvh);


    for (int j = 0; j < num_spheres; j++) {
      float t_new = intersects_sphere(ray, spheres[j]);
      if (t_new < t) { t = t_new; best_j = j; typ = 2; }
    }


}


vec3 trace(Ray ray, inout uint seed, bool importance_sampling, bool direct_light_sampling, bool russian_roulette) {
    vec3 emit = vec3(0.0);
    vec3 trans = vec3(1.0);
    bool last_specular  = true;

    for (int j = 0; j < 4; j++) {
      int typ;
      int best_j;
      float t  = 1.0e34;

      float bvh = 0.0;

      intersect(ray, typ, best_j, t, bvh);

      if (debug == 1) {
        return vec3(0.0, bvh, 0.0);
      }

      if (t >= 1.0e3) {
        emit = vec3(0.0);
        break;
      }

      Material material;
      switch (typ) {
        case 0: material = planes[best_j].material; break;
        case 1: material = triangles[best_j].material; break;
        case 2: material = spheres[best_j].material; break;
        case -1: material = light.material; break;
      }

      vec3 intersection = ray.origin + ray.direction * t;
      vec3 normal;
      switch (typ) {
        case -1: normal = light.normal; break;
        case 0: normal = planes[best_j].normal; break;
        case 1: normal = triangles[best_j].normal; break;
        case 2: normal = normalize(intersection - spheres[best_j].position); break;
      }

      if (material.emissive == 1 && dot(normal, ray.direction) <= 0.0) {
        if (direct_light_sampling) {
            if (last_specular) {
                emit += trans * material.diffuse;
            }
            break;
        }
        emit += trans * material.diffuse;
        break;
      }
        
      
      vec3 brdf = material.diffuse * (1.0 / PI);

      if (direct_light_sampling) {
        vec3 pol = random_point_on_triangle(light, seed);
        vec3 ld = pol - intersection;
        vec3 nld = normalize(ld);
        float dist = length(ld);
        Ray lr;
        lr.origin = intersection + (EPSILON * nld);
        lr.direction = nld;
        lr.inv_direction = 1.0 / lr.direction;

        vec3 nl = light.normal;

        if (dot(normal, nld) > 0. && dot(nl, -nld) > 0. && intersect_shadow(lr, 1e8) >= dist) {
          float area = triangle_area(light);
          float solid_angle = (dot(nl, -nld) * area) / (dist * dist);
          float light_pdf = 1.0 / solid_angle;
          emit += trans * (dot(normal, nld) / light_pdf) * brdf * light.material.diffuse;
        }

      }
   
      float r0 = next_float_lcg(seed);
      if (r0 < material.refl) {
        last_specular = true;
        ray.direction = reflect(ray.direction, normal);
        ray.origin = intersection;
        ray.origin += ray.direction * EPSILON;
        ray.inv_direction = 1.0 / ray.direction;
        trans *= material.diffuse;
      } else {
        last_specular = false;
        float r0 = next_float_lcg(seed);
        float cos_i;
        float pdf;
        if (importance_sampling) {
          ray.direction = local_to_world(diffuse_reflection_cos(seed), normal); 
          ray.origin = intersection + ray.direction * 0.01;
          ray.inv_direction = vec3(1.0)/ray.direction;
          cos_i = dot(ray.direction, normal);
          pdf = cos_i / PI;
        } else {
          ray.direction = local_to_world(diffuse_reflection(seed), normal);
          ray.origin = intersection + ray.direction * 0.01;
          ray.inv_direction = vec3(1.0)/ray.direction;
          cos_i = dot(ray.direction, normal);
          pdf = 1.0 / (2.0 * PI);
        }

        if (russian_roulette) {
          float r0 = next_float_lcg(seed);
          float survival = clamp(0.1, 1.0, max(max(trans.x, trans.y),trans.z));
          if (r0 < survival) {
            trans /= survival;
          } else {
            break;
          }
        }
        if (importance_sampling) {
          trans *=  PI * brdf; 
        } else {
          trans *= (cos_i * ( 1.0 / pdf)) * brdf;
        }
     }


    }

    return emit;
}


void main() {

    //imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(vec3(0.0), 1.0)); 
    uint idx = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * imageSize(img).x;

    if (frame_num == 1) {
        accum[idx] = vec3(0.0);
        imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(vec3(0.0), 1.0)); 
    }


    // screen is now 512 by 512. which fits in 9 bits. 
    uint seed = (gl_GlobalInvocationID.x) | (gl_GlobalInvocationID.y << 9) |  (frame_num << 18);
    // we want to decoralate pixels. Hashes are very suited for this
    seed = wang_hash(seed);

    float r0 = next_float_lcg(seed);
    float r1 = next_float_lcg(seed);
    vec2 uv = (vec2(gl_GlobalInvocationID.xy) + vec2(r0, r1)) / imageSize(img);

    Ray ray = generate_ray(uv);
  
    // TODO make these constants?
    //
    // TODO clamp reflective services
    
    bool importance_sampling = true;
    bool direct_light_sampling = true; gl_GlobalInvocationID.x > 255;
    bool russian_roulette = true;
    vec3 color = trace(ray, seed, importance_sampling, direct_light_sampling, russian_roulette);

    accum[idx] +=  color;
    vec3 outCol = accum[idx] / float(frame_num);
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(outCol, 1.0));


}
"]
#[allow(dead_code)]
struct Dummy;

use bvh::aabb::{AABB, Bounded};
use bvh::ray::Intersection;
use bvh::ray::Ray;
use bvh::flat_bvh;
use bvh::bounding_hierarchy::BHShape;
use obj::*;
use obj::raw::object::Polygon;

fn aabb_to_aabb(aabb: AABB) -> ty::AABB {
    ty::AABB {
        _dummy0: [0;4],
        min: [aabb.min.x, aabb.min.y, aabb.min.z],
        max: [aabb.max.x, aabb.max.y, aabb.max.z],
    }
}

pub fn node_to_node(node: flat_bvh::FlatNode) -> ty::Node {
    ty::Node {
        _dummy0: [0;4],
        _dummy1: [0;4],
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
    fn set_bh_node_index(&mut self, index: usize) {
        // do nothing, just here for dummy
    }

    fn bh_node_index(&self) -> usize {
        // nothing. just here for dummy
        0
    }

    fn intersect(&self, ray: &Ray) -> Intersection {
        Intersection {
          distance: 0.0,
          u: 0.0,
          v: 0.0,
        }
    }
}

/* 
impl FromRawVertex for ty::Triangle {
    fn process(vertices: Vec<(f32, f32, f32, f32)>,
               _: Vec<(f32, f32, f32)>,
               polygons: Vec<Polygon>)
               -> ObjResult<(Vec<Self>, Vec<u16>)> {
        // Convert the vertices to `Point3`s.
        let points = vertices
            .into_iter()
            .collect::<Vec<_>>();

        // Estimate for the number of triangles, assuming that each polygon is a triangle.
        let mut triangles = Vec::with_capacity(polygons.len());
        {
            let mut push_triangle = |indices: &Vec<usize>| {
                let mut indices_iter = indices.iter();
                let anchor = points[*indices_iter.next().unwrap()];
                let mut second = points[*indices_iter.next().unwrap()];
                for third_index in indices_iter {
                    let third = points[*third_index];

                    triangles.push(ty::Triangle{
                        p1: [anchor.0, anchor.1, anchor.2],
                        p2: [second.0, second.1, second.2],
                        p3: [ third.0,  third.1,  third.2],
                        material: ty::Material{
                            diffuse: [0.7, 0.2, 0.7],
                            emissive: 0,
                            refl: 0.0,
                            _dummy0: [0;8],
                        },
                        _dummy0: [0;4],
                        _dummy1: [0;4],
                        _dummy2: [0;4],
                        _dummy3: [0;4],
                    });
                    second = third;
                }
            };

            // Iterate over the polygons and populate the `Triangle`s vector.
            for polygon in polygons.into_iter() {
                match polygon {
                    Polygon::P(ref vec) => push_triangle(vec),
                    Polygon::PT(ref vec) |
                    Polygon::PN(ref vec) => {
                        push_triangle(&vec.iter().map(|vertex| vertex.0).collect())
                    }
                    Polygon::PTN(ref vec) => {
                        push_triangle(&vec.iter().map(|vertex| vertex.0).collect())
                    }
                }
            }
        }
        Ok((triangles, Vec::new()))
    }
}*/

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
        println!("{:?}", up);
        println!("{:?}", right);
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
// shade: evaluate material at every t  (And do we direct_light_samplingd a shadow ray, and do we direct_light_samplingd to quit or not (russian))
//  -> shadow rays
//  -> path continuationrays
// connect   (will only trace shadow rays)     (is the only one that plot to the screen. Only shadow rays contribute)  (though not true for mirrors)
// jump back to to extend
// compaction can be done with atomic counter
