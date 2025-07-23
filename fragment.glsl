precision highp float;

varying vec2 pos;

void main() {
    gl_FragColor = vec4(pos.x, 0., 1., 1.);
}
