// Adapted from wgpu v30.0.1, examples/features/src/boids/draw.wgsl
// (https://github.com/gfx-rs/wgpu/tree/v30.0.1/examples/features/src/boids).
// Copyright (c) 2025 The gfx-rs developers. Licensed under either of the
// Apache License, Version 2.0 or the MIT license, at your option.

// The simulation uses isotropic world units with X in ±halfWidth and Y in ±1.
struct SimParams {
    deltaT: f32,
    rule1Distance: f32,
    rule2Distance: f32,
    rule3Distance: f32,
    rule1Scale: f32,
    rule2Scale: f32,
    rule3Scale: f32,
    halfWidth: f32,
};
@group(0) @binding(0) var<uniform> params: SimParams;

@vertex
fn main_vs(
    @location(0) particle_pos: vec2<f32>,
    @location(1) particle_vel: vec2<f32>,
    @location(2) position: vec2<f32>,
) -> @builtin(position) vec4<f32> {
    let angle = -atan2(particle_vel.x, particle_vel.y);
    let pos = vec2<f32>(
        position.x * cos(angle) - position.y * sin(angle),
        position.x * sin(angle) + position.y * cos(angle)
    );
    return vec4<f32>((pos + particle_pos) / vec2<f32>(params.halfWidth, 1.0), 0.0, 1.0);
}

@fragment
fn main_fs() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}
