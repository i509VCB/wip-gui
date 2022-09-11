#version 100

precision mediump float;

uniform mat4 matrix;
uniform vec2 size;

attribute vec4 position;
attribute vec2 coordinates;

// The local position within the normalized pixel coordinates.
varying vec2 local_position;

void main() {
    local_position = coordinates;
    gl_Position = matrix * vec4(position.xyz, 1.0);
}
