shader_type canvas_item;

uniform float vignette_intensity = 0.4;
uniform float vignette_opacity: hint_range(0.0, 1.0) = 0.5;
uniform vec4 vignette_color: source_color = vec4(0.0, 0.0, 0.0, 1.0);
uniform sampler2D screen_texture : hint_screen_texture, repeat_disable, filter_nearest;

float vignette(vec2 uv) {
	uv *= 1.0 - uv.xy;
	float vignette = uv.x * uv.y * 15.0;
	return pow(vignette, vignette_intensity * vignette_opacity);
}

void fragment() {
	vec4 color = texture(screen_texture, SCREEN_UV);
	vec4 text = texture(TEXTURE, UV);

	text *= vignette_color;
	text *= 1.0 - vignette(UV);

	COLOR = vec4(text.rgb * color.rgb, text.a);
}
