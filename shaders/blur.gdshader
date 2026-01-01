shader_type canvas_item;

uniform float kernel = 16.0;
uniform float sigma = 3.0;
uniform sampler2D screen_texture : hint_screen_texture, repeat_disable, filter_nearest;

float hash3(float x, float y, float z) {
    return fract(sin(dot(vec3(z, y, x), vec3(12.9898, 78.233, 43.706))) * 43758.5453123);
}

void fragment() {
	COLOR = vec4(0.0, 0.0, 0.0, 0.0);

	vec2 shift;
	for (float i = -kernel; i <= kernel; i++) {
		shift = sigma
			* vec2(cos(TAU * i / kernel), sin(TAU * i / kernel))
			* hash3(i, SCREEN_UV.x, SCREEN_UV.y)
			/ vec2(textureSize(screen_texture, 0));
		COLOR += texture(screen_texture, SCREEN_UV + shift);
	}

	COLOR.rgb = COLOR.rgb / COLOR.a;
	COLOR.a = 1.0;
}
