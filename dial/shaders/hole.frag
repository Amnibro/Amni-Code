#version 450
layout(location = 0) in vec2 in_uv;
layout(set = 3, binding = 0) uniform Hole { vec4 a; vec4 h0; vec4 h1; vec4 m; };
layout(location = 0) out vec4 o;
uint hu(uvec2 v) { v = v * 1664525u + 1013904223u; v.x += v.y * 1664525u; v.y += v.x * 1664525u; v ^= v >> 16u; v.x += v.y * 1664525u; v.y += v.x * 1664525u; v ^= v >> 16u; return v.x ^ v.y; }
float hh(vec2 p) { return float(hu(uvec2(ivec2(floor(p)) + 65536))) * 2.3283064e-10; }
float vn(vec2 p) { vec2 i = floor(p), f = fract(p); f = f * f * (3.0 - 2.0 * f); return mix(mix(hh(i), hh(i + vec2(1.0, 0.0)), f.x), mix(hh(i + vec2(0.0, 1.0)), hh(i + 1.0), f.x), f.y); }
float fbm(vec2 p) { float s = 0.0, w = 0.5; for (int k = 0; k < 5; k++) { s += w * vn(p); p = p * 2.03 + vec2(17.1, 9.2); w *= 0.5; } return s; }
float star(vec2 p, float cell, float t, float seed) { vec2 g = floor(p / cell); vec2 f = p / cell - g; float r = hh(g + seed); vec2 c = vec2(hh(g + seed + 31.0), hh(g + seed + 57.0)) * 0.8 + 0.1; float d = length((f - c) * cell); return smoothstep(0.87, 1.0, r) * (0.7 + 0.3 * sin(t * (0.8 + r * 2.7) + r * 91.0)) * (exp(-d * d * 1.3) * 1.7 + exp(-d * 0.55) * 0.12); }
vec3 sky(vec2 p, float t) {
	vec2 q = p / a.y; float n = fbm(q * 2.1 + vec2(t * 0.003, 0.0)); float w = fbm(q * 4.7 + n * 1.9 - vec2(0.0, t * 0.005));
	vec3 c = vec3(0.010, 0.016, 0.036) + vec3(0.05, 0.028, 0.10) * smoothstep(0.35, 0.85, n) + vec3(0.0, 0.065, 0.085) * smoothstep(0.5, 0.95, w) * n + vec3(0.13, 0.065, 0.02) * smoothstep(0.7, 0.95, n * w * 1.5);
	vec3 s = vec3(1.0, 0.94, 0.84) * (star(p, 19.0, t, 0.0) * 0.55 + star(p, 37.0, t, 101.0) + star(p, 83.0, t, 211.0) * 1.6) + vec3(0.6, 0.75, 1.0) * star(p, 53.0, t, 307.0) * 0.8;
	vec3 paper = vec3(0.958, 0.938, 0.882) + vec3(0.02, 0.018, 0.01) * (n - 0.5) - vec3(0.05, 0.06, 0.08) * smoothstep(0.55, 0.95, w) * 0.4;
	return mix(c + s, paper, a.w);
}
vec3 disk(float x, float ph, float t, float sp) {
	if (x < 1.3 || x > 5.9) return vec3(0.0);
	float pa = ph - t * 1.4 * pow(x, -1.5) * sp;
	float tur = fbm(vec2(x * 4.3, 0.0) + vec2(cos(pa), sin(pa)) * 2.2);
	float e = smoothstep(1.3, 1.75, x) * (1.0 - smoothstep(3.4, 5.9, x)) * pow(x / 1.7, -1.5);
	vec3 tc = mix(mix(vec3(1.0, 0.96, 0.88), vec3(1.0, 0.64, 0.24), smoothstep(1.5, 2.8, x)), vec3(0.62, 0.16, 0.05), smoothstep(2.8, 5.5, x));
	return tc * e * (0.45 + 1.1 * tur * tur) * pow(1.0 - 0.15 * cos(ph) * sp, 2.0) * 0.85;
}
vec3 ink(vec3 c, vec3 e) { float l = clamp(dot(e, vec3(0.3, 0.5, 0.2)) * 1.3, 0.0, 0.92); return mix(c, vec3(0.60, 0.40, 0.11) * (0.55 + 0.45 * l), l); }
vec3 put(vec3 c, vec3 e) { return a.w > 0.5 ? ink(c, e) : c + e * 1.2 / (1.0 + 0.4 * dot(e, vec3(0.3, 0.5, 0.2))); }
void main() {
	vec2 p = in_uv * a.xy; float t = a.z; float sp = m.z; float tilt = m.y; bool over = m.x > 0.5;
	vec4 hs[2] = vec4[2](h0, h1);
	vec2 sp2 = p;
	for (int k = 0; k < 2; k++) { vec4 h = hs[k]; if (h.w < 0.01) continue; vec2 d = p - h.xy; float re = h.z * 2.7 * h.w; sp2 -= d * re * re / (dot(d, d) + h.z * h.z * 0.05); }
	vec3 c = over ? vec3(0.0) : sky(sp2, t);
	float cov = 0.0;
	for (int k = 0; k < 2; k++) {
		vec4 h = hs[k]; if (h.w < 0.01) continue;
		vec2 d = p - h.xy; float r = length(d) + 0.001; float rh = h.z; float xr = r / rh; float ang = atan(d.y, d.x);
		vec2 dd = vec2(d.x, d.y / tilt); float rho = length(dd) / rh; float ph = atan(dd.y, dd.x);
		float mask = smoothstep(0.985, 1.0, xr);
		vec3 ring = vec3(1.0, 0.9, 0.72) * 2.4 * exp(-pow((xr - 1.035) / 0.03, 2.0)) * (1.0 - 0.1 * cos(ang) * sp) * h.w;
		vec3 near = d.y >= 0.0 ? disk(rho, ph, t, sp) * h.w : vec3(0.0);
		if (over) { c = put(c, (near + ring) * (a.w > 0.5 ? 0.0 : 1.0)); cov = max(cov, (1.0 - mask) * h.w); continue; }
		bool top = d.y < 0.0; float tb = smoothstep(-0.7, 0.7, d.y / rh);
		vec3 halo = xr > 1.0 ? disk(1.3 + (xr - 1.07) * mix(2.4, 4.8, tb), ang, t, sp) * mix(0.85, 0.35, tb) * smoothstep(1.0, 1.08, xr) * h.w : vec3(0.0);
		vec3 far = top && xr > 1.0 ? disk(rho, ph, t, sp) * h.w : vec3(0.0);
		c = put(c, halo + far + vec3(1.0, 0.62, 0.28) * 0.10 * exp(-max(xr - 1.0, 0.0) / 2.2) * h.w);
		c = mix(c, c * mask, h.w);
		c = put(c, (near + ring) * (a.w > 0.5 ? 1.0 : 0.0));
	}
	if (over && a.w > 0.5) { o = vec4(0.0, 0.0, 0.0, cov); return; }
	o = over ? vec4(c, cov) : vec4(c, 1.0);
}
