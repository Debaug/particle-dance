struct Transformation {
    center: vec2f,
    scale: f32,
    angle: f32,
    matrix: mat3x3f,
}

@group(0) @binding(0) var<storage> transformations: array<Transformation>;

@vertex
fn vertex(@location(0) point: vec2f) -> @builtin(position) vec4f {
    return vec4f(point, 0.0, 1.0);
}

@fragment
fn fragment(@builtin(position) point: vec4f) -> @location(0) vec4f {
    return vec4f(1.0);
}
