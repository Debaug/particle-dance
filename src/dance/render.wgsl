struct Transformation {
    center: vec2f,
    scale: f32,
    angle: f32,
    color: vec4f,
    matrix: mat3x3f,
}

@group(0) @binding(0) var<storage> transformations: array<Transformation>;

struct Vertex {
    @builtin(position) position: vec4f,
    @location(0) point: vec2f,
}

@vertex
fn vertex(@location(0) point: vec2f) -> Vertex {
    var v: Vertex;
    v.position = vec4f(point, 0.0, 1.0);
    v.point = point;
    return v;
}

@fragment
fn fragment(@location(0) point: vec2f) -> @location(0) vec4f {
    var totalLength: f32 = 0.0;
    var color = vec4f(0.0);
    for (var i: u32 = 0; i < arrayLength(&transformations); i++) {
        let s = (point.xy - transformations[i].center);
        let t = 1. / dot(s, s);
        // return vec4f(t, 0., 0., 1.);
        color += t * transformations[i].color;
        totalLength += t;
    }
    return color / totalLength;
}
