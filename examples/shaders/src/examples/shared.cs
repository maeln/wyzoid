#version 450

const uint LEN = 128;
const uint GRP = 64;

layout(local_size_x = GRP, local_size_y = 1, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer Data { int data[]; };

shared int p[GRP];

void main() {
  p[gl_LocalInvocationID.x] = data[gl_GlobalInvocationID.x];

  memoryBarrierShared();
  barrier();

  int val = p[gl_LocalInvocationID.x];

  if(gl_GlobalInvocationID.x > 0) {
    if(gl_LocalInvocationID.x > 0) {
      val += p[gl_LocalInvocationID.x-1];
    } else {
      val += data[gl_GlobalInvocationID.x-1];
    }
  }

  if(gl_GlobalInvocationID.x < LEN) {
    if(gl_LocalInvocationID.x < (GRP-1)) {
      val += p[gl_LocalInvocationID.x+1];
    } else {
      val += data[gl_GlobalInvocationID.x+1];
    }
  }

  memoryBarrier();
  barrier();

  data[gl_GlobalInvocationID.x] = val;
}