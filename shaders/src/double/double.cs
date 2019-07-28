#version 450

layout(local_size_x = 128, local_size_y = 1, local_size_z = 1) in;

layout(std430, binding = 0) buffer Data { 
    vec4 data[]; 
};

void main() {
    uint idx = gl_GlobalInvocationID.x;
    float r = sqrt(data.x * data.z);
    float c = sin(data.y);
    data[idx] = vec4(r*c, c, 0, 0);
}
