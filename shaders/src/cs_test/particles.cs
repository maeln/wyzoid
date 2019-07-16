#version 430 core

layout(local_size_x = 128, local_size_y = 1, local_size_z = 1) in;
layout(std140, binding = 0) buffer PositionBuffer { vec4 positions[]; };
layout(std430, binding = 1) buffer TtlBuffer { float ttls[]; };

layout(binding = 0) uniform UBO{
float time;
float dt;
float speed;
} ubo;

float random(vec2 st) { return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123); }

void main(void)
{
	uint id = gl_GlobalInvocationID.x;
	float ttl = ttls[id];
	vec4 pos = positions[id];

	float workGroupNum = float(gl_NumWorkGroups.x);
	float workGroupSize = float(gl_WorkGroupSize.x);
	float workGroupLen = workGroupNum * workGroupSize;
	float workGroupLoc = float(gl_WorkGroupID.x);
	float localId = float(gl_LocalInvocationID.x);

	float i = float(id);
	float s = workGroupLen;

	float mx = 128.0;
	float my = 32.0;

	float x = i / (mx * my);
	float z = mod(i, mx);
	float y = mod(i / mx, my);

	vec3 gl = vec3(x, y, z) / 16.0;

	/*
	const float mgx = floor(sqrt(workGroupNum));
	float gx = mod(workGroupLoc, mgx);
	float gy = floor(workGroupLoc / mgx);
	vec3 g = vec3(gx, 0.0, gy);

	const float mlx = floor(sqrt(workGroupSize));
	float lx = mod(localId, mlx);
	float ly = floor(localId / mlx);
	vec3 l = vec3(lx, 0.0, ly);

	vec3 gl = g + l / sqrt(workGroupSize);
	*/

	/*
	vec3 localCoord = vec3(localId / (workGroupSize * workGroupSize), 0.0, mod(localId, workGroupSize));
	vec3 groupCoord = vec3(workGroupLoc / (workGroupNum * workGroupNum), 0.0, mod(workGroupLoc, workGroupNum));
	vec3 globalCoord = groupCoord + localCoord;
	*/

	// for now, never ending particles
	/*
	float nttl = ttl - dt;
	if (nttl <= 0.0) {
		npos = vec4(0.0, 0.0, 0.0, 1.0);
		nttl = 2.5;
	}
	*/

	positions[id] = vec4(gl, 1.0);
	ttls[id] = ttl;
}
