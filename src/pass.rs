//! The world drawn in one pass on the GPU, where Bevy takes two (Hajun,
//! 2026-11-15: the phone "becomes somewhat hot" played for long, and nothing
//! to be seen was to change for it).
//!
//! Bevy 0.19 draws a camera's world in two render passes: everything solid
//! (`main_opaque_pass_3d`), then everything see-through
//! (`main_transparent_pass_3d`: the sea, the clouds, the fountain's water,
//! the punch's range). Both draw into the same picture, at four samples a
//! pixel, with the same depth. A phone's GPU draws a pass a tile at a time
//! in a small memory of its own, so between two passes it has to write every
//! tile out to the phone's memory and read it back in: the picture's four
//! samples a pixel and the depth's, the whole screen, every frame, for
//! nothing that shows; and it works the one-sample picture out twice. A
//! desktop's GPU draws straight into memory and is none the worse for two
//! passes, which is why Bevy has them.
//!
//! Here the two are one pass ([`world_pass`]): the solid things, then the
//! see-through, as before, in the order and with the pipelines Bevy drew
//! them with, so the picture is the same to the last pixel (compared on
//! desktop, drawn both ways: `docs/rendering-and-frame-rate.md`). And at
//! its end the four samples and the depth are let go rather than kept: the
//! one-sample picture made from them is what the UI is drawn on and what
//! goes to the screen, and nothing reads the others again.
//!
//! **What needs the passes apart gets Bevy's two back by itself**
//! ([`apart`]): a material that light passes through
//! (`specular_transmission`, Blender's Transmission: Bevy copies the
//! picture between the two passes for it) and order-independent
//! transparency. **What does not, and has to be looked at before it is
//! added:** anything else that reads the depth or the samples after the
//! world is drawn: a second camera on the same target, depth of field,
//! Bevy's atmosphere, screen-space reflections, a depth prepass read back.
//! None is in the game. [`ONE_PASS`] turns all of this off.
//!
//! Not seen on a phone, Android's or Apple's: what it saves is a
//! tile-drawing GPU's work, and this PC has none to measure it on.

use std::sync::atomic::{AtomicBool, Ordering};

use bevy::camera::{MainPassResolutionOverride, Viewport};
use bevy::core_pipeline::core_3d::{
    AlphaMask3d, Opaque3d, Transparent3d, main_opaque_pass_3d, main_transparent_pass_3d,
};
use bevy::core_pipeline::oit::OrderIndependentTransparencySettings;
use bevy::core_pipeline::oit::resolve::OitResolvePipelineId;
use bevy::core_pipeline::skybox::{SkyboxBindGroup, SkyboxPipelineId};
use bevy::core_pipeline::{Core3d, Core3dSystems};
use bevy::ecs::schedule::{InternedSystemSet, ScheduleCleanupPolicy};
use bevy::pbr::{Transmissive3d, main_transmissive_pass_3d};
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::diagnostic::RecordDiagnostics;
use bevy::render::render_phase::{ViewBinnedRenderPhases, ViewSortedRenderPhases};
use bevy::render::render_resource::{PipelineCache, RenderPassDescriptor, StoreOp};
use bevy::render::renderer::{RenderContext, ViewQuery};
use bevy::render::view::{ExtractedView, ViewDepthTexture, ViewTarget, ViewUniformOffset};
use bevy::render::{Render, RenderApp};

/// Whether the world is drawn in one pass. Off, Bevy draws it in its own
/// two, as it did until 2026-11-15.
pub(crate) const ONE_PASS: bool = true;

/// Whether the frame being drawn had its see-through things drawn with its
/// solid ones ([`world_pass`]), so that [`see_through_pass`] has nothing
/// left to do. Written and read on the render thread, one after the other.
#[derive(Resource, Default)]
struct Together(AtomicBool);

/// Whether Bevy's two passes have been taken out and these put in.
#[derive(Resource)]
struct Swapped;

pub(crate) fn plugin(app: &mut App) {
    if !ONE_PASS {
        return;
    }
    let Some(render) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render
        .init_resource::<Together>()
        .add_systems(Render, swap.run_if(not(resource_exists::<Swapped>)));
}

/// Takes Bevy's two passes out of the camera's schedule and puts these in
/// where they were, once, on the first frame: by then everything the
/// schedule's systems are made of is there, as it is when Bevy first runs
/// it. Both, or neither: if Bevy's are not both there to take out, or
/// either does not come out, the world is drawn as Bevy draws it.
fn swap(world: &mut World) {
    world.insert_resource(Swapped);
    let swapped = world.try_schedule_scope(Core3d, |world, schedule| {
        if let Err(error) = schedule.initialize(world) {
            error!("the world is drawn in Bevy's two passes: {error:?}");
            return;
        }
        let has = |schedule: &Schedule, set: InternedSystemSet| {
            schedule.graph().systems_in_set(set).is_ok_and(|systems| systems.len() == 1)
        };
        let solid = main_opaque_pass_3d.into_system_set().intern();
        let see_through = main_transparent_pass_3d.into_system_set().intern();
        if !has(schedule, solid) || !has(schedule, see_through) {
            error!("Bevy's two passes are not where they were: the world is drawn as Bevy draws it");
            return;
        }
        // The systems go and their places stay: whatever Bevy has ordered
        // before or after either is still before or after what takes it.
        let mut out = |pass| {
            schedule
                .remove_systems_in_set(pass, world, ScheduleCleanupPolicy::RemoveSystemsOnly)
                .is_ok_and(|taken| taken == 1)
        };
        match (out(solid), out(see_through)) {
            (true, true) => {
                schedule.add_systems((
                    world_pass
                        .after(main_opaque_pass_3d)
                        .before(main_transmissive_pass_3d)
                        .before(main_transparent_pass_3d)
                        .in_set(Core3dSystems::MainPass),
                    see_through_pass
                        .after(world_pass)
                        .after(main_transmissive_pass_3d)
                        .before(main_transparent_pass_3d)
                        .in_set(Core3dSystems::MainPass),
                ));
            }
            // One came out and the other did not: Bevy's own goes back.
            (solid_out, see_through_out) => {
                error!("a pass could not be taken out: the world is drawn as Bevy draws it");
                if solid_out {
                    schedule.add_systems(
                        main_opaque_pass_3d
                            .before(main_transmissive_pass_3d)
                            .before(main_transparent_pass_3d)
                            .in_set(Core3dSystems::MainPass),
                    );
                }
                if see_through_out {
                    schedule.add_systems(
                        main_transparent_pass_3d
                            .after(main_opaque_pass_3d)
                            .after(main_transmissive_pass_3d)
                            .in_set(Core3dSystems::MainPass),
                    );
                }
            }
        }
    });
    if swapped.is_err() {
        error!("no schedule to draw the world in one pass in: it is drawn in two");
    }
}

/// Whether this view's two passes have to stay apart, with everything kept
/// between them: something is drawn between the two that reads the first
/// (`transmissive`: a material light passes through), or the see-through
/// things are sorted on the GPU (`sorted`: order-independent transparency,
/// which has a pass of its own after them).
fn apart(transmissive: bool, sorted: bool) -> bool {
    transmissive || sorted
}

/// Draws the world: everything solid, then, in the same pass, everything
/// see-through, and keeps of it only the picture. Bevy's
/// `main_opaque_pass_3d` with its `main_transparent_pass_3d` drawn in it;
/// where they have to stay apart ([`apart`]) it is the first alone, as Bevy
/// has it, and [`see_through_pass`] is the second.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn world_pass(
    world: &World,
    view: ViewQuery<(
        &ExtractedCamera,
        &ExtractedView,
        &ViewTarget,
        &ViewDepthTexture,
        Option<&SkyboxPipelineId>,
        Option<&SkyboxBindGroup>,
        &ViewUniformOffset,
        Option<&MainPassResolutionOverride>,
        Has<OrderIndependentTransparencySettings>,
    )>,
    opaque_phases: Res<ViewBinnedRenderPhases<Opaque3d>>,
    alpha_mask_phases: Res<ViewBinnedRenderPhases<AlphaMask3d>>,
    transparent_phases: Res<ViewSortedRenderPhases<Transparent3d>>,
    transmissive_phases: Option<Res<ViewSortedRenderPhases<Transmissive3d>>>,
    pipeline_cache: Res<PipelineCache>,
    together: Res<Together>,
    mut ctx: RenderContext,
) {
    let view_entity = view.entity();
    let (
        camera,
        extracted_view,
        target,
        depth,
        skybox_pipeline,
        skybox_bind_group,
        view_uniform_offset,
        resolution_override,
        sorted,
    ) = view.into_inner();
    together.0.store(false, Ordering::Relaxed);
    let (Some(opaque_phase), Some(alpha_mask_phase)) = (
        opaque_phases.get(&extracted_view.retained_view_entity),
        alpha_mask_phases.get(&extracted_view.retained_view_entity),
    ) else {
        return;
    };
    let transmissive = transmissive_phases
        .as_deref()
        .and_then(|phases| phases.get(&extracted_view.retained_view_entity))
        .is_some_and(|phase| !phase.items.is_empty());
    let one = !apart(transmissive, sorted);
    together.0.store(one, Ordering::Relaxed);
    let transparent_phase = transparent_phases
        .get(&extracted_view.retained_view_entity)
        .filter(|phase| one && !phase.items.is_empty());

    let diagnostics = ctx.diagnostic_recorder();
    let diagnostics = diagnostics.as_deref();

    // Cleared, as the first to draw this frame. In one pass, the samples
    // are not kept once the picture is made of them: with no samples (MSAA
    // off) this is the picture itself, and it is kept.
    let mut color = target.get_color_attachment();
    if one && color.resolve_target.is_some() {
        color.ops.store = StoreOp::Discard;
    }
    // Nor, in one pass, is the depth: nothing after the world reads it, and
    // not keeping it leaves it to be cleared by whoever next draws with it.
    let depth_stencil_attachment = Some(depth.get_attachment(if one { StoreOp::Discard } else { StoreOp::Store }));

    let mut render_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("world_pass"),
        color_attachments: &[Some(color)],
        depth_stencil_attachment,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    let pass_span = diagnostics.pass_span(&mut render_pass, "world_pass");

    if let Some(viewport) = Viewport::from_viewport_and_override(camera.viewport.as_ref(), resolution_override) {
        render_pass.set_camera_viewport(&viewport);
    }

    if !opaque_phase.is_empty()
        && let Err(err) = opaque_phase.render(&mut render_pass, world, view_entity)
    {
        error!("Error encountered while rendering the opaque phase {err:?}");
    }

    if !alpha_mask_phase.is_empty()
        && let Err(err) = alpha_mask_phase.render(&mut render_pass, world, view_entity)
    {
        error!("Error encountered while rendering the alpha mask phase {err:?}");
    }

    if let (Some(skybox_pipeline), Some(SkyboxBindGroup(skybox_bind_group))) = (skybox_pipeline, skybox_bind_group)
        && let Some(pipeline) = pipeline_cache.get_render_pipeline(skybox_pipeline.0)
    {
        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &skybox_bind_group.0, &[view_uniform_offset.offset, skybox_bind_group.1]);
        render_pass.draw(0..3, 0..1);
    }

    // The see-through things, over the solid ones, the furthest first.
    if let Some(transparent_phase) = transparent_phase
        && let Err(err) = transparent_phase.render(&mut render_pass, world, view_entity)
    {
        error!("Error encountered while rendering the transparent phase {err:?}");
    }

    pass_span.end(&mut render_pass);
}

/// Draws the see-through things in a pass of their own, where
/// [`world_pass`] has not drawn them in its own: Bevy's
/// `main_transparent_pass_3d`, as it is.
#[allow(clippy::type_complexity)]
fn see_through_pass(
    world: &World,
    view: ViewQuery<(
        &ExtractedCamera,
        &ExtractedView,
        &ViewTarget,
        &ViewDepthTexture,
        Option<&MainPassResolutionOverride>,
        Has<OrderIndependentTransparencySettings>,
        Option<&OitResolvePipelineId>,
    )>,
    transparent_phases: Res<ViewSortedRenderPhases<Transparent3d>>,
    together: Res<Together>,
    mut ctx: RenderContext,
) {
    if together.0.load(Ordering::Relaxed) {
        return;
    }
    let view_entity = view.entity();
    let (camera, extracted_view, target, depth, resolution_override, has_oit, oit_resolve_pipeline_id) =
        view.into_inner();
    let Some(transparent_phase) = transparent_phases.get(&extracted_view.retained_view_entity) else {
        return;
    };
    if transparent_phase.items.is_empty() {
        return;
    }
    let diagnostics = ctx.diagnostic_recorder();
    let diagnostics = diagnostics.as_deref();
    if has_oit {
        // As Bevy has it: without its pipeline the lists are not reset.
        let Some(oit_resolve_pipeline_id) = oit_resolve_pipeline_id else {
            return;
        };
        if world.resource::<PipelineCache>().get_render_pipeline(oit_resolve_pipeline_id.0).is_none() {
            return;
        }
    }
    let mut render_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("main_transparent_pass_3d"),
        color_attachments: &[Some(target.get_color_attachment())],
        depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    let pass_span = diagnostics.pass_span(&mut render_pass, "main_transparent_pass_3d");
    if let Some(viewport) = Viewport::from_viewport_and_override(camera.viewport.as_ref(), resolution_override) {
        render_pass.set_camera_viewport(&viewport);
    }
    if let Err(err) = transparent_phase.render(&mut render_pass, world, view_entity) {
        error!("Error encountered while rendering the transparent phase {err:?}");
    }
    pass_span.end(&mut render_pass);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_passes_stay_apart_only_for_what_reads_between_them() {
        assert!(!apart(false, false), "the game as it is: one pass");
        assert!(apart(true, false), "a material light passes through");
        assert!(apart(false, true), "order-independent transparency");
    }
}
