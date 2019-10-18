#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer ToSin { float tosin[]; };
layout(std430, set = 0, binding = 1) buffer ToCos { float tocos[]; };

#define TAYLOR_ITER 32.0

float mpow(float a, int b) {
  float acc = 1.0;
  for (int i = 0; i < b; ++i) {
    acc *= a;
  }
  return acc;
}

float fact(float x) {
  float acc = 1.0;
  for (float i = 1.0; i <= x; i += 1.0) {
    acc = acc * i;
  }
  return acc;
}

float taylor_sin(float x) {
  float acc = 0.0;
  for (int i = 0; i < TAYLOR_ITER; i += 1) {
    float n = float(i);
    acc += (mpow(-1.0, i) * pow(x, 2.0 * n + 1.0)) / fact(2.0 * n + 1.0);
  }
  return acc;
}

float taylor_cos(float x) {
  float acc = 0.0;
  for (int i = 0; i < TAYLOR_ITER; i += 1) {
    float n = float(i);
    acc += (mpow(-1.0, i) * pow(x, 2.0 * n)) / fact(2.0 * n);
  }
  return acc;
}

void main() {
  uint idx = gl_GlobalInvocationID.x;
  tosin[idx] = taylor_sin(tosin[idx]);
  tocos[idx] = taylor_cos(tocos[idx]);
}