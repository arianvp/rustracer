#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable
layout(set = 0, binding = 0) uniform sampler2D tex;
layout (location = 0) in vec2 position;
layout (location = 0) out vec4 f_color;
void main() {
  f_color = texture(tex, (position+1.0)/2.0);
}
