shader_type canvas_item;

uniform vec3 light_ray = vec3(0.1, -0.9, 0.0);
uniform float depth = 1.0;
uniform float scale = 0.02;
uniform float amplify = 1.5;
uniform int octaves = 4;
uniform vec3 ior = vec3(0.70, 0.75, 0.80);
uniform float timescale = 1.0;

vec2 random2(vec2 st){
    st = vec2(dot(st, vec2(127.1, 311.7)), dot(st, vec2(269.5, 183.3)));
	return -1.0 + 2.0 * fract(sin(st) * 43758.5453123);
}

float noise(vec2 st) {
    vec2 i = floor(st);
    vec2 f = fract(st);
    vec2 u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    return mix(mix(dot(random2(i + vec2(0.0,0.0)), f - vec2(0.0, 0.0)),
                   dot(random2(i + vec2(1.0,0.0)), f - vec2(1.0, 0.0)), u.x),
               mix(dot(random2(i + vec2(0.0,1.0)), f - vec2(0.0, 1.0)),
                   dot(random2(i + vec2(1.0,1.0)), f - vec2(1.0, 1.0)), u.x), u.y);
}

float fbm(vec2 st) {
    float value = 0.0;
    float amplitude = 0.5;
    float frequency = 1.0;
    for (int i = 0; i < octaves; i++) {
        value += amplitude * noise(st * frequency + TIME * timescale);
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

vec4 sample_noise(vec2 p, float a) {
    float eps = 0.1;
    float nL = fbm(p + vec2(-eps,  0.0)) * a;
    float nR = fbm(p + vec2( eps,  0.0)) * a;
    float nT = fbm(p + vec2( 0.0, -eps)) * a;
    float nB = fbm(p + vec2( 0.0,  eps)) * a;
	vec3 n = normalize(vec3(nL - nR, eps * 2.0, nT - nB));
    float v = fbm(p) * a;
    return vec4(n.x, n.y, n.z, v);
}

float sample_caustic(vec2 p, vec3 l, float H, vec3 N, float h, vec3 n, float i) {
	vec3 R = refract(l, N, i);
	if (length(R) < 0.1) R = l;
	vec2 hit_XZ = p.xy - R.xz * (H / R.y);
	float A = length(dFdx(hit_XZ)) * length(dFdy(hit_XZ));

	vec3 r = refract(l, n, i);
	if (length(r) < 0.1) r = l;
	vec2 hit_xz = p.xy - r.xz * (h / r.y);
	float a = length(dFdx(hit_xz)) * length(dFdy(hit_xz));

	return A / max(a, 0.001);
}

void fragment() {
	vec3 l = normalize(light_ray);
	vec2 p = SCREEN_UV.xy / SCREEN_PIXEL_SIZE.xy;

	float H = depth;
	vec3 N = vec3(0.0, 1.0, 0.0);

	vec4 sample = sample_noise(p / scale, amplify);
	float h = depth + sample.w;
	vec3 n = sample.xyz;

	float caustics_x = sample_caustic(p, l, H, N, h, n, ior.x);
	float caustics_y = sample_caustic(p, l, H, N, h, n, ior.y);
	float caustics_z = sample_caustic(p, l, H, N, h, n, ior.z);
	COLOR.rgb = vec3(caustics_x, caustics_y, caustics_z);
}
