#[derive(VulkanoShader)]
#[ty = "compute"]

#[src = "
#version 450

layout(local_size_x = 16, local_size_y = 16) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;
layout(set = 0, binding = 1) uniform readonly Input {
    vec2 center;
    float scale;
    int iter;
};

struct Sphere {
  vec3 position;
  float radius;
};

//layout(std140, set = 0, binding = 2) buffer Spheres { Sphere spheres[]; };
// layout(std140, set = 0, binding = 2) buffer Positions { vec3 positions[]; };
// layout(std140, set = 0, binding = 3) buffer Indices { uvec3 indices[]; };


struct Ray {
  vec3 origin;
  vec3 direction;
};


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

void main() {
    uint seed = wang_hash(wang_hash(gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * imageSize(img).x));
    float val = seed * (1.0 / 4294967269.0);

    imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(vec3(val), 1.0));
}
"]

#[allow(dead_code)]
struct Dummy;
