shader_type spatial;
render_mode skip_vertex_transform, unshaded;

#define TEXTURE_SIZE 1024
#define BUFFER_SIZE 1024
#define ALPHA_THRESHOLD 0.5

uniform sampler2DArray texture_array: source_color, repeat_disable, filter_nearest;
uniform sampler2D compute_texture: repeat_disable, filter_nearest;
uniform ivec4[BUFFER_SIZE] uniform_buffer;

varying flat float page;

void vertex() {
	int start_texcoord_id = uniform_buffer[INSTANCE_ID].x;
	int end_texcoord_id = uniform_buffer[INSTANCE_ID].y + 1;
	int texcoord_id = start_texcoord_id + int(TIME) % (end_texcoord_id - start_texcoord_id);

	ivec2 iuv = ivec2((2 * texcoord_id) % TEXTURE_SIZE, (2 * texcoord_id) / TEXTURE_SIZE);
	vec4 texcoord = texelFetch(compute_texture, iuv + ivec2(0, 0), 0);
	vec4 mics = texelFetch(compute_texture, iuv + ivec2(1, 0), 0);

	UV = UV * texcoord.zw + texcoord.xy;
	page = mics.x;

	VERTEX = (MODELVIEW_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec4 col = textureLod(texture_array, vec3(UV, page), 0.0);

	if (col.a < ALPHA_THRESHOLD) {
		discard;
	}

	ALBEDO = col.rgb;
}
