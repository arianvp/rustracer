#version 450
layout (location = 0) in vec2 position;
layout (location = 0) out vec2 out_position;
out gl_PerVertex { vec4 gl_Position; };
void main() {
  out_position = position;
  gl_Position = vec4(position.xy, 0.0, 1.0);
}
