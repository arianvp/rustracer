#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

// layout (binding = 1) uniform sampler2D samplerColor;

layout (location = 0) in vec2 position;

layout (location = 0) out vec4 f_color;

void main() 
{
 // f_color = vec4(position.x,position.y,0.0, 1.0);
 // outFragColor = texture(samplerColor, inUV);






    vec2 c = (position - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

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

}
