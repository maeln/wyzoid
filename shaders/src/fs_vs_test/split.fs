#version 330 core
out vec4 FragColor;

in vec2 TexCoords;

uniform vec2 resolution;

uniform sampler2D scenebuffer;
uniform sampler2D backbuffer;

void main()
{
	vec2 uv = gl_FragCoord.xy / resolution.xy;
    if(uv.x > 0.5) {
        FragColor = vec4(sqrt(texture(scenebuffer, uv).rgb), 1.0);
    } else {
        FragColor = vec4(sqrt(texture(backbuffer, uv).rgb), 1.0);
    }
}