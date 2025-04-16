shader_type spatial;
render_mode skip_vertex_transform, unshaded;

#define MAX_BUFFER_SIZE 1024
#define ALPHA_THRESHOLD 0.5
uniform sampler2DArray texture_array: source_color, repeat_disable, filter_nearest;
uniform sampler2D bake_texture: repeat_disable, filter_nearest;
uniform uvec4[MAX_BUFFER_SIZE] head_buffer;
uniform uint[MAX_BUFFER_SIZE] color_buffer;
uniform uint tick;

varying flat float PAGE;
varying flat vec4 BLEND;

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

	// Decode color_buffer
	uint color = color_buffer[INSTANCE_ID];
	BLEND = vec4(
		float((color >> 28u) & 0xFu) / float(0xFu),
		float((color >> 24u) & 0xFu) / float(0xFu),
		float((color >> 20u) & 0xFu) / float(0xFu),
		float((color >> 16u) & 0xFu) / float(0xFu)
	);
	COLOR = vec4(
		float((color >> 12u) & 0xFu) / float(0xFu),
		float((color >> 08u) & 0xFu) / float(0xFu),
		float((color >> 04u) & 0xFu) / float(0xFu),
		float((color >> 00u) & 0xFu) / float(0xFu)
	);

	VERTEX = (MODELVIEW_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec4 col = textureLod(texture_array, vec3(UV, PAGE), 0.0);

	if (col.a < ALPHA_THRESHOLD) {
		discard;
	}

	ALBEDO = mix(col.rgb, COLOR.rgb, BLEND.rgb);
}
