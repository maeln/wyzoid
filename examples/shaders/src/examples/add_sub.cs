#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer Buffer1 { float buf1[]; };
layout(std430, set = 0, binding = 1) buffer Buffer2 { float buf2[]; };

// Classic 2D noise function
float random (vec2 st) {
    return fract(sin(dot(st.xy,
                         vec2(12.9898,78.233)))*
        43758.5453123);
}

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf1[idx] = buf1[idx] - 1.0;
    buf2[idx] = buf2[idx] + 1.0;
}