#version 450
layout(location = 0) in vec2 in_uv;
layout(set = 2, binding = 0) uniform sampler2D src;
layout(set = 2, binding = 1) uniform sampler2D glo;
layout(set = 3, binding = 0) uniform Post { vec4 a; vec4 b; };
layout(location = 0) out vec4 o;
vec3 tap(vec2 u) { return texture(src, u).rgb; }
vec3 box(vec2 u, vec2 e) { return (tap(u + vec2(-e.x, -e.y)) + tap(u + vec2(e.x, -e.y)) + tap(u + vec2(-e.x, e.y)) + tap(u + e)) * 0.25; }
void main() {
	vec2 e = a.xy; float md = a.z;
	if (md < 0.5) { vec3 c = box(in_uv, e); float l = max(c.r, max(c.g, c.b)); o = vec4(c * smoothstep(b.y, b.y + 0.5, l), 1.0); return; }
	if (md < 1.5) { o = vec4(box(in_uv, e), 1.0); return; }
	if (md < 2.5) { vec3 c = tap(in_uv) * 4.0 + (tap(in_uv + vec2(e.x, 0.0)) + tap(in_uv - vec2(e.x, 0.0)) + tap(in_uv + vec2(0.0, e.y)) + tap(in_uv - vec2(0.0, e.y))) * 2.0 + tap(in_uv + e) + tap(in_uv - e) + tap(in_uv + vec2(e.x, -e.y)) + tap(in_uv + vec2(-e.x, e.y)); o = vec4(c / 16.0 * a.w, 1.0); return; }
	vec2 q = in_uv - 0.5; float d2 = dot(q, q); float ca = d2 * 0.003 * (1.0 - b.w);
	vec3 c = vec3(texture(src, in_uv - q * ca).r, texture(src, in_uv).g, texture(src, in_uv + q * ca).b) + texture(glo, in_uv).rgb * a.w;
	c *= 1.0 - d2 * mix(0.85, 0.2, b.w);
	uvec2 g = uvec2(in_uv * 4096.0) * uvec2(1664525u, 22695477u) + uvec2(uint(b.x * 50.0) * 2654435761u); g ^= g.yx >> 13u; g *= 1103515245u;
	c += (float((g.x ^ g.y) >> 8u) * 5.9604645e-8 - 0.5) * b.z;
	vec3 k = max(c - 0.8, 0.0);
	o = vec4(min(c, vec3(0.8)) + 0.2 * (1.0 - exp(-k / 0.2)), 1.0);
}
