#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

// layout (binding = 1) uniform sampler2D samplerColor;

layout (location = 0) in vec2 position;

layout (location = 0) out vec4 f_color;

void main() 
{
  f_color = vec4(1.0,0.0,0.0, 1.0);
 // outFragColor = texture(samplerColor, inUV);
}
