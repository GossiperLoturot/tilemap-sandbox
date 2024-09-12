shader_type spatial;
render_mode skip_vertex_transform, unshaded;

#define MAX_BUFFER_SIZE 1024
#define ALPHA_THRESHOLD 0.5

#define SHADOW_DIR vec3(0.5, 0.0, 0.0)
#define SHADOW_BASE vec3(0.0, 0.0, -0.03125)
#define SHADOW vec4(0.0, 0.0, 0.0, 0.25)

uniform sampler2D screen_texture: hint_screen_texture, repeat_disable, filter_nearest;

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
	int tick_offset = head_buffer[INSTANCE_ID].w;

	int texcoord_id = start_texcoord_id;
	if (step_tick > 0) {
		int step_id = (tick - tick_offset) / step_tick;
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

	VERTEX = (MODEL_MATRIX * vec4(VERTEX.x, VERTEX.y, 0.0, 1.0)).xyz + SHADOW_DIR * (mat3(MODEL_MATRIX) * VERTEX).z + SHADOW_BASE;
	VERTEX = (VIEW_MATRIX * vec4(VERTEX, 1.0)).xyz;
}

void fragment() {
	vec4 col = textureLod(texture_array, vec3(UV, PAGE), 0.0);

	if (col.a < ALPHA_THRESHOLD) {
		discard;
	}

	vec4 src = textureLod(screen_texture, SCREEN_UV, 0.0);
	ALBEDO = mix(src.rgb, SHADOW.rgb, SHADOW.a);
}
