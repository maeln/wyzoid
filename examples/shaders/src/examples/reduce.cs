#version 450

#define LSIZE 64

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer Data { float data[]; };
layout(std430, set = 0, binding = 1) buffer DataOut { float data_out[]; };

// shared float red[LSIZE];

void main() {
    uint i = gl_GlobalInvocationID.x;
    uint l = gl_LocalInvocationID.x;
    uint w = gl_WorkGroupID.x;

    uint wg = gl_WorkGroupSize.x;

    // uint data_per_th = data.length() / (LSIZE * wg);
    
    // u = (w * wg) + ()

    uint u = i*64;
    uint next = min((i+1)*64, 256);
    float acc = 0.0;
    for(uint n=u; n<next; n++) {
        acc += data[n];
    }

    data_out[i] = acc;
}