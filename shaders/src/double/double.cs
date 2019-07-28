#version 450

layout(local_size_x = 128, local_size_y = 1, local_size_z = 1) in;

layout(std430, binding = 0) buffer Data { 
    float data[]; 
};

void main() {
    uint idx = gl_GlobalInvocationID.x;
    data[idx] = sqrt(data[idx]);
}
