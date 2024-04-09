shader_type spatial;
render_mode unshaded;

#define BUFFER_SIZE 1024

uniform float[BUFFER_SIZE] page_buffer;
uniform vec4[BUFFER_SIZE] texcoord_buffer;

uniform sampler2DArray textures: filter_nearest;

varying flat float page;

void vertex() {
	page = page_buffer[INSTANCE_ID];
	
	vec2 offset = texcoord_buffer[INSTANCE_ID].xy;
	vec2 scale = texcoord_buffer[INSTANCE_ID].zw;
	UV = UV * scale + offset;
}

void fragment() {
	vec4 col = texture(textures, vec3(UV, page));
	ALBEDO = col.rgb;
	ALPHA = col.a;
}
