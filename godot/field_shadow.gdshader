shader_type spatial;
render_mode skip_vertex_transform, unshaded;

#define BUFFER_SIZE 1024
#define ALPHA_THRESHOLD 0.5

#define SHADOW_DIR vec3(0.5, 0.0, 0.0)
#define SHADOW_BASE vec3(0.0, 0.0, -0.03125)
#define SHADOW vec4(0.0, 0.0, 0.0, 0.25)

uniform sampler2D screen_texture: hint_screen_texture, repeat_disable, filter_nearest;

uniform sampler2DArray texture_array: source_color, repeat_disable, filter_nearest;
uniform float[BUFFER_SIZE] page_buffer;
uniform vec4[BUFFER_SIZE] texcoord_buffer;

varying flat float page;

void vertex() {
	page = page_buffer[INSTANCE_ID];

	vec2 offset = texcoord_buffer[INSTANCE_ID].xy;
	vec2 scale = texcoord_buffer[INSTANCE_ID].zw;
	UV = UV * scale + offset;

	VERTEX = (MODEL_MATRIX * vec4(VERTEX.x, VERTEX.y, 0.0, 1.0)).xyz + SHADOW_DIR * (mat3(MODEL_MATRIX) * VERTEX).z + SHADOW_BASE;
	VERTEX = (VIEW_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec4 col = textureLod(texture_array, vec3(UV, page), 0.0);

	if (col.a < ALPHA_THRESHOLD) {
		discard;
	}

	vec4 src = textureLod(screen_texture, SCREEN_UV, 0.0);
	ALBEDO = mix(src.rgb, SHADOW.rgb, SHADOW.a);
}
