shader_type spatial;
render_mode unshaded;

#define BUFFER_SIZE 1024
#define ALPHA_THRESHOLD 0.5

uniform sampler2DArray texture_array: filter_nearest;
uniform float[BUFFER_SIZE] page_buffer;
uniform vec4[BUFFER_SIZE] texcoord_buffer;

varying flat float page;

void vertex() {
	page = page_buffer[INSTANCE_ID];

	vec2 offset = texcoord_buffer[INSTANCE_ID].xy;
	vec2 scale = texcoord_buffer[INSTANCE_ID].zw;
	UV = UV * scale + offset;
}

void fragment() {
	vec4 col = texture(texture_array, vec3(UV, page));

	if (col.a < ALPHA_THRESHOLD) {
		discard;
	}

	ALBEDO = col.rgb;
}
