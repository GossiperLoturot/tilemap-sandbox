shader_type spatial;
render_mode unshaded;

uniform sampler2D texture0;
// uniform sampler2D texture1;
// uniform sampler2D texture2;
// uniform sampler2D texture3;
// uniform sampler2D texture4;
// uniform sampler2D texture5;
// uniform sampler2D texture6;
// uniform sampler2D texture7;

void vertex() {
	// Called for every vertex the material is visible on.
}

void fragment() {
	ALBEDO = texture(texture0, UV).rgb;
}
