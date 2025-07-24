#version 330

precision highp float;

varying vec2 pos;

void main() {
    vec4 c1 = vec4(0.5, 0.1, 0.9, 1.);
    vec4 c2 = vec4(0.1, 0.8, 0.7, 1.);
    gl_FragColor = mix(c1, c2, pos.x);
}
