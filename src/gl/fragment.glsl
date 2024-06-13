#version 140

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

ivec4 u32d4(uint data);
ivec3 u32d3(uint data);
float read_load_pixel(uint x, uint y);
vec4 bar(inout vec4 O, vec2 U);
vec4 bar_pixel(inout vec4 O, vec2 U, uint bar_index);
vec4 bar_history_pixel(inout vec4 O, vec2 U, uint bar_index);
void gague(inout vec4 O, vec2 uv, vec2 center, int radius, int line_width, vec3 color, float angle);
float read_pixel(uint index, uint ptr);
vec4 gague_circle(inout vec4 O, vec2 U, uint gauge_index);
vec3 u32color(uint data);

uniform sensors {
  uint width;
  uint time;
  uint gauge_count;
  uint gauge_value[6];
  uint gauge_color[6];
  uint load_ptr;
  uint load_count;
  uint load_color[24];
  uint load[2048];
};

uniform sampler2D tex;

/*
  ██████╗ ██████╗ ███╗   ██╗███████╗████████╗ █████╗ ███╗   ██╗████████╗███████╗
 ██╔════╝██╔═══██╗████╗  ██║██╔════╝╚══██╔══╝██╔══██╗████╗  ██║╚══██╔══╝██╔════╝
 ██║     ██║   ██║██╔██╗ ██║███████╗   ██║   ███████║██╔██╗ ██║   ██║   ███████╗
 ██║     ██║   ██║██║╚██╗██║╚════██║   ██║   ██╔══██║██║╚██╗██║   ██║   ╚════██║
 ╚██████╗╚██████╔╝██║ ╚████║███████║   ██║   ██║  ██║██║ ╚████║   ██║   ███████║
  ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═══╝   ╚═╝   ╚══════╝
*/

float TAU = 6.28318530718;

uint text_start = 256u;

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
uint gauge_start = width - gauge_dist * gauge_groups;
uint gauge_space = 4u;

vec4 gague(inout vec4 O, vec2 U) {
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
 ████████╗███████╗██╗  ██╗████████╗
 ╚══██╔══╝██╔════╝╚██╗██╔╝╚══██╔══╝
    ██║   █████╗   ╚███╔╝    ██║
    ██║   ██╔══╝   ██╔██╗    ██║
    ██║   ███████╗██╔╝ ██╗   ██║
    ╚═╝   ╚══════╝╚═╝  ╚═╝   ╚═╝
*/

#define _MINUS  TXT(p, uvec3(     262144,    268451841,  16778240));
#define _PERIOD TXT(p, uvec3(          0,       393240,         0));
#define _COLON  TXT(p, uvec3(          0,  2349215744u,         1));
#define _0      TXT(p, uvec3(  272596992,    151274514, 266355009));
#define _1      TXT(p, uvec3( 1073872896,  4278192128u,         1));
#define _2      TXT(p, uvec3(  274825216,    285508628, 274743873));
#define _3      TXT(p, uvec3(  270565376,    285492240, 249578561));
#define _4      TXT(p, uvec3( 2148270080u,   570462210,  33587136));
#define _5      TXT(p, uvec3( 2418262016u,   151266320, 252723777));
#define _6      TXT(p, uvec3( 2420080640u,   151266320, 251675201));
#define _7      TXT(p, uvec3(  268451840,    822543360,   3146560));
#define _8      TXT(p, uvec3(  272334848,    285492241, 249578561));
#define _9      TXT(p, uvec3(  268664832,    285492241, 132129857));
#define _       CURSOR.x += 8;
vec3 TXT_COL = vec3(0.0);

ivec2 CURSOR_START = ivec2(0), CURSOR = ivec2(0);

void TXT(vec2 p, uvec3 g) {
  _ int x = int(p.x) - CURSOR.x, y = CURSOR.y - int(p.y), b = x * 14 + y - 16;
  if(x > 0 && x < 8 && y >= 0 && y < 14 && b >= 0)
    TXT_COL += vec3((g[b / 32] >> (b & 31)) & 1u);
}
void _NUM(vec2 p, float n) {
  if(n < 0.) {
    _MINUS n = -n;
  }
  for(int i = 6, k = 100000000, m = int(round(n * 100.0)); i > -3; i--, k /= 10) {
    int d = m >= k || i <= 0 ? int(m / k) % 10 : -1;
    if(i == -1)
      _PERIOD if(d == 0)
      _0 if(d == 1)
      _1 if(d == 2)
      _2 if(d == 3)
      _3 if(d == 4)
      _4 if(d == 5)
      _5 if(d == 6)
      _6 if(d == 7)
      _7 if(d == 8)
      _8 if(d == 9)
      _9 if(i == 0 && (int(m) % 100) == 0)
      break;
  }
}

#define NUM(value) _NUM(p,value);

vec4 draw_text(vec4 O) {
  CURSOR_START = CURSOR = ivec2(float(text_start), 17.);
  vec2 p = gl_FragCoord.xy;
  ivec3 time = u32d3(time);

  NUM(time.x);
  _COLON;
  NUM(time.y);
  _COLON;
  NUM(time.z);

  O += vec4(mix(vec3(.2, .2, .2), vec3(.1, .8, .1), TXT_COL), 1.0);
  return O;
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
  /* if(U.x > 184.0 || U.y > 24.0) {
    f_color = vec4(1. - U.x / 1920, 0.0, 0.0, 1.0);
    return;
  } */
  O = bar(O, U);
  O = gague(O, U);
  O = draw_text(O);

  // O = texture(tex, v_tex_coords);

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

float read_load_pixel(uint ptr, uint index) {
  uint page_index = index * 64u + uint(floor(ptr / 4u));
  uint byte_index = ptr % 4u;
  ivec4 page = u32d4(load[page_index]);
  return float(page[byte_index]) / 255.0;
}

/* float read_load_pixel(uint index, uint ptr) {
  ivec4 page = u32d4(load[int(ptr * 64u + floor(index / 4u))]);
  return page[index % 4u] / 255.0;
}
 */

float read_pixel(uint index, uint ptr) {
  ivec4 page = u32d4(load[int(index * 64u + floor(float(ptr) / 4.0))]);
  return float(page[ptr % 4u]) / 255.0;
}
