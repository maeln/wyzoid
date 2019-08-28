#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer Data { 
  float data[]; 
};

layout(std430, set = 0, binding = 1) buffer Viz { 
  vec4 viz[]; 
};

// fifth-order polynomial approximation of Turbo based on:
// https://observablehq.com/@mbostock/turbo
vec3 turbo(float x) {
    float r = 0.1357 + x * ( 4.5974 - x * ( 42.3277 - x * ( 130.5887 - x * ( 150.5666 - x * 58.1375 ))));
	float g = 0.0914 + x * ( 2.1856 + x * ( 4.8052 - x * ( 14.0195 - x * ( 4.2109 + x * 2.7747 ))));
	float b = 0.1067 + x * ( 12.5925 - x * ( 60.1097 - x * ( 109.0745 - x * ( 88.5066 - x * 26.8183 ))));
    return vec3(r,g,b);
}

void main() {
  uint idx = gl_GlobalInvocationID.x;
  viz[idx] = vec4(turbo(data[idx]), 1.0);
}
