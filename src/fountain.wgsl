// The fountain's moving water (`fountain::FlowingWater`): see-through, with
// streaks of white running along it. Each sheet of it is laid out with `uv.x`
// going round the fountain, 0 to 1, and `uv.y` along the way the water runs,
// in metres; the colour's alpha fades it in where it starts and out where it
// ends. The streaks move by the frame's time alone, so nothing is sent to the
// GPU from one frame to the next to keep them going.
//
// Unlit: water spilling in the sun is mostly the light it throws back.

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::{globals, view},
}
#ifdef TONEMAP_IN_SHADER
#import bevy_core_pipeline::tonemapping::tone_mapping
#endif

struct FlowingWater {
    // The water between the streaks, and in `a` how solid it is.
    tint: vec4<f32>,
    // The streaks, and in `a` how solid they are at most.
    streak: vec4<f32>,
    // x: how fast the water runs, in metres a second. y: how many streaks there
    // are round the fountain. z: how long a streak and the gap after it are,
    // in metres. w: how wide a streak is, as a share of the room between two.
    flow: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> water: FlowingWater;

// A number from 0 to 1 that looks nothing like `n`, and is the same for it
// every time.
fn scatter(n: f32) -> f32 {
    return fract(sin(n * 91.345 + 47.853) * 43758.547);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
#ifdef VERTEX_UVS_A
    let uv = in.uv;
#else
    let uv = vec2<f32>(0.0);
#endif
    let around = uv.x * water.flow.y;
    let lane = floor(around);
    let one = scatter(lane);
    let two = scatter(lane + 31.0);
    // Each lane's streaks are a length of their own and run at a speed of
    // their own, so that no two lanes keep step.
    let length = water.flow.z * (0.7 + 0.6 * one);
    let speed = water.flow.x * (0.85 + 0.3 * two);
    let along = fract((uv.y - globals.time * speed) / length + one * 7.0);
    // 0 down the middle of the lane, 1 at either side of it.
    let across = abs(fract(around) - 0.5) * 2.0;
    let width = water.flow.w * (0.6 + 0.8 * two);
    let core = 1.0 - smoothstep(width * 0.25, width, across);
    // A streak runs the whole way, brighter where a pulse of water is passing.
    let pulse = smoothstep(0.0, 0.25, along) * (1.0 - smoothstep(0.5, 1.0, along));
    let streak = core * (0.3 + 0.7 * pulse) * (0.6 + 0.4 * one);
    var color = mix(water.tint, water.streak, streak);
    // Looked at along it rather than through it, a sheet of water is more
    // solid, which shows its shape.
    let to_eye = normalize(view.world_position.xyz - in.world_position.xyz);
    let glance = 1.0 - abs(dot(normalize(in.world_normal), to_eye));
    color.a = min(color.a * (1.0 + 1.5 * glance * glance), 1.0);
#ifdef VERTEX_COLORS
    color.a = color.a * in.color.a;
#endif
#ifdef TONEMAP_IN_SHADER
    color = tone_mapping(color, view.color_grading);
#endif
    return color;
}
