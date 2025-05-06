shader_type spatial;
render_mode skip_vertex_transform, unshaded;

#define MAX_BUFFER_SIZE 1024
#define ALPHA_THRESHOLD 0.5
uniform sampler2DArray texture_array: source_color, repeat_disable, filter_nearest;
uniform sampler2D bake_texture: repeat_disable, filter_nearest;
uniform uvec4[MAX_BUFFER_SIZE] head_buffer;
uniform uint tick;

varying flat float PAGE;

void vertex() {
	// Decode bake_texture and head_buffer

	uint start_texcoord_id = head_buffer[INSTANCE_ID].x;
	uint end_texcoord_id = head_buffer[INSTANCE_ID].y;
	uint step_tick = head_buffer[INSTANCE_ID].z & 0xFFFFu;
	uint is_loop = head_buffer[INSTANCE_ID].z >> 16u & 0x1u;
	uint tick_offset = head_buffer[INSTANCE_ID].w;

	uint texcoord_id = start_texcoord_id;
	if (step_tick > 0u) {
		uint step_id = (tick - tick_offset) / step_tick;
		uint step_size = end_texcoord_id - start_texcoord_id;

		if (is_loop != 0u) {
			texcoord_id += step_id % step_size;
		} else {
			texcoord_id += min(step_id, step_size - 1u);
		}
	}

	ivec2 size = textureSize(bake_texture, 0);
	ivec2 iuv = ivec2(
		int((texcoord_id * 2u) % uint(size.x)),
		int((texcoord_id * 2u) / uint(size.y))
	);
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
