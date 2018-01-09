#version 450

layout(local_size_x = 16, local_size_y = 16) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;
layout(set = 0, binding = 1) uniform readonly Input {
    vec2 center;
    float scale;
    int iter;
};

void main() {
    vec2 uv = vec2(gl_GlobalInvocationID.xy) / imageSize(img);
    vec2 c = (uv - vec2(0.5)) * scale - vec2(1.0, 0.0);
    vec2 z = vec2(0.0, 0.0);
    float i;
    float step = 1.0 / iter;
    for (i = 0; i < 1.0; i += step) {
        z = vec2(
            z.x * z.x - z.y * z.y + c.x,
            z.y * z.x + z.x * z.y + c.y
        );

        if (length(z) > 4.0) {
            break;
        }
    }
    imageStore(img, ivec2(gl_GlobalInvocationID.xy), vec4(vec3(i), 1.0));
}
