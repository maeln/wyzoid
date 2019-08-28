#version 450

#define WIDTH 256.0
#define HEIGHT 256.0

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(std430, set = 0, binding = 0) buffer Data { 
  float data[]; 
};

float random (in vec2 st) {
    return fract(sin(dot(st.xy,
                         vec2(12.9898,78.233)))*
        43758.5453123);
}

// Based on Morgan McGuire @morgan3d
// https://www.shadertoy.com/view/4dS3Wd
float noise (in vec2 st) {
    vec2 i = floor(st);
    vec2 f = fract(st);

    // Four corners in 2D of a tile
    float a = random(i);
    float b = random(i + vec2(1.0, 0.0));
    float c = random(i + vec2(0.0, 1.0));
    float d = random(i + vec2(1.0, 1.0));

    vec2 u = f * f * (3.0 - 2.0 * f);

    return mix(a, b, u.x) +
            (c - a)* u.y * (1.0 - u.x) +
            (d - b) * u.x * u.y;
}

float fbm (in vec2 st) {
    // Initial values
    float value = 0.0;
    float amplitude = 0.5;
    
    // Loop of octaves
    for (int i = 0; i < 8; i++) {
        value += amplitude * noise(st);
        st *= 3.;
        amplitude *= .5;
    }
    return value;
}

void main() {
  vec3 idx = gl_GlobalInvocationID;
  vec2 uv = vec2(idx.x, idx.y) / vec2(WIDTH, HEIGHT);
  data[uint(idx.y*WIDTH + idx.x)] = fbm(uv);
}
