#version 100

precision mediump float;

// TODO: Replace with vertex attribute to allow for gradients
uniform vec4 u_color;

void main() {
    gl_FragColor = u_color;
}
