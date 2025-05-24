struct Transformation {
    center: vec2f,
    scale: f32,
    angle: f32,
    color: vec4f,
    matrix: mat3x3f,
}

@group(0) @binding(0) var<storage> transformations: array<Transformation>;
@group(1) @binding(0) var<storage, read_write> points: array<vec2f>;

@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) id: vec3u) {
    let point = points[id.x];
    let hash = bitcast<u32>(point.x) ^ bitcast<u32>(point.y) ^ id.x;
    let idx = hash % (arrayLength(&transformations));
    let transformation = transformations[idx].matrix;
    points[id.x] = (transformation * vec3f(point, 1.0)).xy;
}
