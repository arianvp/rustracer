
pub mod vs {
    #[derive(VulkanoShader)]
    #[ty = "vertex"]
    #[src = "
#version 450

layout (location = 0) in vec2 position;
layout (location = 0) out vec2 out_position;

out gl_PerVertex
{
	vec4 gl_Position;
};

void main() 
{
	out_position = position;
        gl_Position = vec4(position.xy, 0.0, 1.0);
}
"]
    #[allow(dead_code)]
    struct Dummy;
}

pub mod fs {
    #[derive(VulkanoShader)]
    #[ty = "fragment"]

    #[src = "

#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable


layout (location = 0) in vec2 position;
// layout (binding = 0, rgba8) uniform readonly image2D to_draw;

layout (location = 0) out vec4 f_color;

void main() 
{
 // f_color = vec4(position.x,position.y,0.0, 1.0);
 // outFragColor = texture(samplerColor, inUV);

    //vec3 rgb = imageLoad(to_draw, ivec2(position)).rgb;
    //f_color = vec4(rgb.rgb, 1.0);*/


    vec2 c = (position) * 2.0 - vec2(1.0, 0.0);

    vec2 z = vec2(0.0, 0.0);
    float i;
    for (i = 0.0; i < 1.0; i += 0.005) {
        z = vec2(
            z.x * z.x - z.y * z.y + c.x,
            z.y * z.x + z.x * z.y + c.y
        );

        if (length(z) > 4.0) {
            break;
        }
    }

    vec4 to_write = vec4(vec3(i), 1.0);
    f_color = to_write;

}"]
    #[allow(dead_code)]
    struct Dummy;
}
