#version 450

const uint LEN = 128;
const uint GRP = 64;

layout(local_size_x = GRP, local_size_y = 1, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer Data { float data[]; };

shared float p[GRP];
shared float m[GRP];

void main() {
  p[gl_LocalInvocationID.x] = data[gl_GlobalInvocationID.x];
  m[gl_LocalInvocationID.x] = data[gl_GlobalInvocationID.x];
  memoryBarrierShared();
  barrier();

  for (int i = 0; i < GRP; ++i) {
    uint nextId = (gl_LocalInvocationID.x + 1) % GRP;
    float nextValue = p[nextId];
    float myValue = p[gl_LocalInvocationID.x];
    barrier();
    // Compare and swap:
    if (nextValue < myValue) {
      p[gl_LocalInvocationID.x] = nextValue;
      m[gl_LocalInvocationID.x] = myValue;
    }
    memoryBarrierShared();
    barrier();
  }

  data[gl_GlobalInvocationID.x] = p[gl_LocalInvocationID.x];
}