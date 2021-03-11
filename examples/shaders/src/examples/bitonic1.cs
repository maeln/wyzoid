#version 450

const uint GRP = 32;

layout(local_size_x = GRP, local_size_y = 1, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer Data { float data[]; };
layout(std430, set = 0, binding = 1) buffer DataOut { float output_data[]; };

shared float local_buffer[GRP];

void main() {
  uint idx  = gl_LocalInvocationID.x;
  local_buffer[idx] = data[gl_GlobalInvocationID.x];
  memoryBarrierShared();
  barrier();

  for(uint l = 2; l<=GRP; l *= 2) {
      for(uint j = l/2; j>0; j /= 2) {
          uint sib = idx ^ j;
          float val = local_buffer[idx];
          float sib_val = local_buffer[sib];

          if(sib > idx) {
              if( (((idx & l) == 0) && val > sib_val) || (((idx & l) != 0) && val < sib_val) ) {
                memoryBarrierShared();
                barrier();

                local_buffer[idx] = sib_val;
                local_buffer[sib] = val;

                memoryBarrierShared();
                barrier();
              }
          }
      }
  }

  output_data[gl_GlobalInvocationID.x] = local_buffer[idx];
}