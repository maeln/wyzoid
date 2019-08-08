#version 450

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer Data { float data[]; };
layout(std430, set = 0, binding = 1) buffer Key { float key[]; };

#define TAYLOR_ITER 32.0

float fact(float x) {
    float acc = 1.0;
    for(float i=1.0; i<=x; i+=1.0) {
        acc = acc * i;
    }
    return acc;
}

float taylor_sin(float x) {
    float acc = 0.0;
    for(float i=0.0; i<TAYLOR_ITER; i+=1.0) {
        acc += (pow(-1.0, i) * pow(x, 2.0 * i + 1.0)) / fact(2.0 * i + 1.0);
    }
    return acc;
}

void main() {
    uint idx = gl_GlobalInvocationID.x;
    // data[idx] = taylor_sin(data[idx]);
    data[idx] = 0.0;
}