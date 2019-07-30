#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(std430, binding = 0) buffer Data { 
    vec4 data[]; 
};

void main() {
    uint idx = gl_GlobalInvocationID.x;
    vec4 orig = data[idx];
    
    data[idx] = vec4(orig.x*2.0, orig.x*orig.y, orig.x+orig.y, orig.z-orig.x);
}
