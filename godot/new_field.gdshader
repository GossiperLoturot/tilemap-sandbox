shader_type spatial;
render_mode skip_vertex_transform, unshaded;

#define MAX_BUFFER_SIZE 1024
#define ALPHA_THRESHOLD 0.5

uniform sampler2DArray texture_array: source_color, repeat_disable, filter_nearest;
uniform sampler2D bake_texture: repeat_disable, filter_nearest;
uniform ivec4[MAX_BUFFER_SIZE] head_buffer;
uniform int tick;

varying flat float PAGE;

void vertex() {
	// Decode bake_texture and head buffer

	int start_texcoord_id = head_buffer[INSTANCE_ID].x;
	int end_texcoord_id = head_buffer[INSTANCE_ID].y;
	int step_tick = head_buffer[INSTANCE_ID].z & 0xFFFF;
	int is_loop = head_buffer[INSTANCE_ID].z >> 16 & 0x1;
	int is_local_tick = head_buffer[INSTANCE_ID].z >> 17 & 0x1;
	int tick_offset = head_buffer[INSTANCE_ID].w;

	int texcoord_id = start_texcoord_id;
	if (step_tick > 0) {
		int step_id = (tick - tick_offset * is_local_tick) / step_tick;
		int step_size = end_texcoord_id - start_texcoord_id;

		if (is_loop != 0) {
			texcoord_id += step_id % step_size;
		} else {
			texcoord_id += min(step_id, step_size - 1);
		}
	}

	ivec2 size = textureSize(bake_texture, 0);
	ivec2 iuv = ivec2((texcoord_id * 2) % size.x, (texcoord_id * 2) / size.y);
	vec4 texcoord = texelFetch(bake_texture, iuv + ivec2(0, 0), 0);
	vec4 misc = texelFetch(bake_texture, iuv + ivec2(1, 0), 0);

	UV = UV * texcoord.zw + texcoord.xy;
	PAGE = misc.x;

	VERTEX = (MODELVIEW_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec4 col = textureLod(texture_array, vec3(UV, PAGE), 0.0);

	if (col.a < ALPHA_THRESHOLD) {
		discard;
	}

	ALBEDO = col.rgb;
}
