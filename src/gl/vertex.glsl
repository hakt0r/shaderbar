#version 460

in vec2 position;
in vec2 tex_coords;
out vec2 v_tex_coords;

const mat4 IDENTITY = mat4(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0);

void main() {
  v_tex_coords = tex_coords;
  gl_Position = IDENTITY * vec4(position, 0.0, 1.0);
}