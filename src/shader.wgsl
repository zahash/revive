struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VsOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

@vertex
fn vs_main( in: VsIn ) -> VsOut {
    var out: VsOut;
    out.color = in.color;
    out.clip_position = vec4<f32>(in.position, 1.0);
    return out;
}

struct FsOut {
    @location(0) color: vec4<f32>,
}

@fragment
fn fs_main(in: VsOut) -> FsOut {
    var out: FsOut;
    out.color = vec4<f32>(in.color, 1.0);
    return out;
}
