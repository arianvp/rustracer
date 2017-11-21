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

layout(set = 0, binding = 0) uniform sampler2D tex;

layout (location = 0) in vec2 position;
// layout (binding = 0, rgba8) uniform readonly image2D to_draw;

layout (location = 0) out vec4 f_color;

void main() 
{
    f_color = texture(tex, (position+1.0)/2.0);

}"]
    #[allow(dead_code)]
    struct Dummy;
}
