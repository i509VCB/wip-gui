#version 100

precision highp float;

uniform vec4 color;

// The size of the shape as a ratio.
//
// One value must always be 1.0. The other value may be greater than 1.0 to create a rectangle.
uniform vec2 size;

// The radius in a fraction.
//
// A value of 0.0 indicates that a quad should be drawn.
// A value of 1.0 indicates that the radius of the curves should occupy the entire width or height (whichever is shorter).
// Values between 0.0 and 1.0 will result in a quad with curved edges.
uniform float radius;

// The thickness of the shape.
//
// A thickness of 1.0 means the shape will be solid.
// A thickness of 0.0 means the shape will be invisible.
//
// Note: With smaller values for radius, the inner border might be a right angle.
//
// If the thickness is greater or equal to the radius, then the inside of the shape will be solid.
uniform float thickness;

// The fade applied to edges of the shape.
//
// This value is used to smooth the edges of the shape and remove jagged edges.
//
// A fade of 0.0 will not smooth the shape.
// High fade values will cause the shape to become invisible.
uniform float fade;

// The local position in normalized pixel coordinates.
//
// This is a value between 0 and 1.
varying vec2 local_position;

// SDF for a rectangle.
//
// The `half_perimeter` is half the width and height of the rectangle in normalized pixel coordinates.
// The `radius` is the radius of the corners in normalized pixel coordinates.
//
// Inspired by https://www.shadertoy.com/view/WtdSDs
float _distance(vec2 position, vec2 size, float radius) {
    vec2 q = abs(position) - size + radius;
    return length(max(q, 0.0)) - radius;
}

void main() {
    // Subtract half the fade to ensure the rectangle is not clipped.
    vec2 size = size - (fade / 2.0);

    // Scale the position based on the size of the rectangle in normalized pixel coordinates.
    // Then subtract half the size to center the position within normalized pixel coordinates.

    float distance = _distance(local_position * size, size, radius);
    float smoothed_alpha = 1.0 - smoothstep(0.0, fade, distance);
    float border_alpha = 1.0 - smoothstep(thickness - fade, thickness, abs(distance));

    const vec4 CLEAR = vec4(0.0);

    gl_FragColor = mix(CLEAR, mix(CLEAR, color, border_alpha), smoothed_alpha);

    if (gl_FragColor.a == 0.0) {
        discard;
    }
}
