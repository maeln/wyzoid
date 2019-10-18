#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer Data { float data[]; };
layout(std430, set = 0, binding = 1) buffer DataOut { float data_out[]; };

shared float local_val[64];

void main() {
    uint i = gl_GlobalInvocationID.x;
    uint l = gl_LocalInvocationID.x;
    uint wg = gl_WorkGroupSize.x;

    local_val[l] = data[i];
    memoryBarrierShared();
    barrier();

    for(int len=1; len<wg; len <<= 1) {
        bool direction = ((l & (len<<1)) != 0);
        for(int inc=len; inc > 0; inc>>=1) {
            uint j = l ^ inc;
            float a = local_val[l];
            float b = local_val[j];

            bool smaller = (b < a) || (a == b && j < l);
            bool swap = smaller ^^ (j < l) ^^ direction;

            memoryBarrierShared();
            barrier();
            local_val[l] = swap ? b : a;
            memoryBarrierShared();
            barrier();
        }
    }

    data_out[i] = local_val[l];
}