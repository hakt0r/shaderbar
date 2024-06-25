#version 460

in vec2 v_tex_coords;
out vec4 f_color;

/*
 ██╗  ██╗███████╗ █████╗ ██████╗ ███████╗██████╗
 ██║  ██║██╔════╝██╔══██╗██╔══██╗██╔════╝██╔══██╗
 ███████║█████╗  ███████║██║  ██║█████╗  ██████╔╝
 ██╔══██║██╔══╝  ██╔══██║██║  ██║██╔══╝  ██╔══██╗
 ██║  ██║███████╗██║  ██║██████╔╝███████╗██║  ██║
 ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝
*/

float read_pixel(uint index, uint ptr);
ivec3 u32d3(uint data);
ivec4 u32d4(uint data);
vec3 u32color(uint data);
vec4 bar_history_pixel(inout vec4 O, vec2 U, uint bar_index);
vec4 bar_pixel(inout vec4 O, vec2 U, uint bar_index);
vec4 bar(inout vec4 O, vec2 U);
vec4 draw_icon(vec4 O, vec2 U);
uint align_char(uint char);
vec4 gague_circle(inout vec4 O, vec2 U, uint gauge_index);
void gague(inout vec4 O, vec2 uv, vec2 center, int radius, int line_width, vec3 color, float angle);

uniform sensors {
  uint width;
  uint gauge_count;
  uint gauge_value[6];
  uint gauge_color[6];
  uint load_ptr;
  uint load_count;
  uint load_color[24];
  uint load[2048];
};

uniform sampler2D font;

/*
  ██████╗ ██████╗ ███╗   ██╗███████╗████████╗ █████╗ ███╗   ██╗████████╗███████╗
 ██╔════╝██╔═══██╗████╗  ██║██╔════╝╚══██╔══╝██╔══██╗████╗  ██║╚══██╔══╝██╔════╝
 ██║     ██║   ██║██╔██╗ ██║███████╗   ██║   ███████║██╔██╗ ██║   ██║   ███████╗
 ██║     ██║   ██║██║╚██╗██║╚════██║   ██║   ██╔══██║██║╚██╗██║   ██║   ╚════██║
 ╚██████╗╚██████╔╝██║ ╚████║███████║   ██║   ██║  ██║██║ ╚████║   ██║   ███████║
  ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝   ╚══════╝
*/

float TAU = 6.28318530718;

float bar_dim = 0.8;
float bar_max = 5.0;
float bar_height = 24. / 16.;

/*
  ██████╗  █████╗ ██╗   ██╗ ██████╗ ███████╗
 ██╔════╝ ██╔══██╗██║   ██║██╔════╝ ██╔════╝
 ██║  ███╗███████║██║   ██║██║  ███╗█████╗
 ██║   ██║██╔══██║██║   ██║██║   ██║██╔══╝
 ╚██████╔╝██║  ██║╚██████╔╝╚██████╔╝███████╗
  ╚═════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚══════╝
*/

uint gauge_radius = 9u;
uint gauge_dist = 28u;
uint gauge_groups = 3u;
uint gauge_space = 4u;

vec4 gague(inout vec4 O, vec2 U) {
  uint gauge_start = width - gauge_dist * gauge_groups;
  uint x = uint(U.x), y = uint(U.y);
  float start = float(gauge_start);
  bool is_not_gauge = x < gauge_start;
  bool is_between_gauge = (width - x) % gauge_dist < gauge_space || (width - x) % gauge_dist > gauge_dist - gauge_space;
  if(is_not_gauge || is_between_gauge)
    return O;
  uint gauge_index = (width - x) / gauge_dist * 2u;
  O = gague_circle(O, U, gauge_index);
  O = gague_circle(O, U, gauge_index + 1u);
  return O;
}

vec4 gague_circle(inout vec4 O, vec2 U, uint gauge_index) {
  float gauge_index_f = float(gauge_index);
  vec3 color = u32d3(gauge_color[gauge_index]);
  float red = color.r / 255.0;
  float green = color.g / 255.0;
  float blue = color.b / 255.0;
  float angle = (TAU / 255) * gauge_value[gauge_index];
  int line_width = gauge_index % 2u == 0u ? 5 : 1;
  int radius = gauge_index % 2u == 0u ? 7 : 4;
  vec2 center = vec2(float(width - gauge_dist * uint(floor(gauge_index_f / 2.)) - gauge_dist / 2u), 11.5);
  vec2 pos = U - center;
  float dist = length(pos);
  float lw2 = float(line_width) / 2.0;
  float alpha = atan(pos.y, -pos.x) + TAU / 4.0;
  alpha += alpha < 0.0 ? TAU : 0.0;

  float edge0 = float(radius) + lw2;
  float edge1 = float(radius) - lw2;

  float blend = 1.2;
  float inside = smoothstep(edge0 - blend, edge0 + blend, dist);
  float outside = smoothstep(edge1 - blend, edge1 + blend, dist);
  float withinAngle = step(alpha, angle);

  float antialias = (1.0 - inside) * outside * withinAngle;

  vec4 finalColor = vec4(red, green, blue, antialias);
  return mix(O, finalColor, finalColor.a);
}

/*
 ██████╗  █████╗ ██████╗
 ██╔══██╗██╔══██╗██╔══██╗
 ██████╔╝███████║██████╔╝
 ██╔══██╗██╔══██║██╔══██╗
 ██████╔╝██║  ██║██║  ██║
 ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
*/

vec4 bar(inout vec4 O, vec2 U) {
  bool is_not_bar = U.x > bar_max + 256.0; //|| U.y > load_count;
  bool is_bar_pixel = U.x <= bar_max;
  uint bar_index = uint(U.y * (load_count / 24.0));
  if(is_not_bar)
    return O;
  if(is_bar_pixel)
    return bar_pixel(O, U, bar_index);
  else
    return bar_history_pixel(O, U, bar_index);
}

vec3 white = vec3(1.0, 1.0, 1.0);

vec4 bar_pixel(inout vec4 O, vec2 U, uint bar_index) {
  float value = read_pixel(bar_index, load_ptr);
  float bar_width = bar_max * value;
  if(U.x < bar_max - bar_width)
    return O;
  return mix(O, vec4(white, 1.0), value);
}

vec4 bar_history_pixel(inout vec4 O, vec2 U, uint bar_index) {
  uint index = uint(U.x - bar_max);
  float value = read_pixel(bar_index, (256u + load_ptr - uint(index)) % 256u);
  float fade = 1. - float(index) / 256.;
  return mix(O, vec4(U.x / 256., U.y / 24., 1., 1.), value * fade * bar_dim);
}

/*
 ███╗   ███╗ █████╗ ██╗███╗   ██╗
 ████╗ ████║██╔══██╗██║████╗  ██║
 ██╔████╔██║███████║██║██╔██╗ ██║
 ██║╚██╔╝██║██╔══██║██║██║╚██╗██║
 ██║ ╚═╝ ██║██║  ██║██║██║ ╚████║
 ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝
*/

void main() {
  vec2 U = gl_FragCoord.xy;
  vec4 O = vec4(0.0, 0.0, 0.0, 0.0);
  O = bar(O, U);
  O = gague(O, U);
  f_color = O;
  return;
}

/*
 ████████╗ ██████╗  ██████╗ ██╗     ███████╗
 ╚══██╔══╝██╔═══██╗██╔═══██╗██║     ██╔════╝
    ██║   ██║   ██║██║   ██║██║     ███████╗
    ██║   ██║   ██║██║   ██║██║     ╚════██║
    ██║   ╚██████╔╝╚██████╔╝███████╗███████║
    ╚═╝    ╚═════╝  ╚═════╝ ╚══════╝╚══════╝
*/

ivec4 u32d4(uint data) {
  return ivec4((data >> 24) & 0xFFu, (data >> 16) & 0xFFu, (data >> 8) & 0xFFu, data & 0xFFu);
}

ivec3 u32d3(uint data) {
  return ivec3((data >> 24) & 0xFFu, (data >> 16) & 0xFFu, (data >> 8) & 0xFFu);
}

vec3 u32color(uint data) {
  return vec3((data >> 24) & 0xFFu, (data >> 16) & 0xFFu, (data >> 8) & 0xFFu) / 255.0;
}

float read_pixel(uint index, uint ptr) {
  ivec4 page = u32d4(load[int(index * 64u + floor(float(ptr) / 4.0))]);
  return float(page[ptr % 4u]) / 255.0;
}
