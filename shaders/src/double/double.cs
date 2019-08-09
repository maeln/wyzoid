#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer Data { float data[]; };
layout(std430, set = 0, binding = 1) buffer Key { float key[]; };

vec4 process(vec4 data, float id) {
  vec4 o = vec4(0);
  for (float i = 0; i < 64.0; i += 1.0) {
    o += (data * i) / (id + 1.0);
    o *= sin(i / 100.0);
  }

  return o;
}

void main() {
  uint idx = gl_GlobalInvocationID.x;
  key[idx] = 1.0;
}
