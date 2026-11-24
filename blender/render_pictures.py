"""Renders the bag's picture of each of a list of models, from the .glb the
game loads, as the pieces of the shop were pictured by the rig kept in
each of Hajun's .blend files (`glTF_not_exported`): 256 px square, a 50 mm
camera turned 60.3 degrees down from straight overhead and 32.7 round, fitted
to the model's corners drawn 18 parts in a hundred further out from its
middle, on a clear background, lit by a plain sky alone (the rig's sun is
in a collection that is not rendered). Nothing is saved but the pictures.

    blender --background --factory-startup --python blender/render_pictures.py -- in.glb out.png [in.glb out.png ...]

Written 2026-11-25 for the shop's 76 new buildings (`docs/bag.md`): their
.blend files are Hajun's, and some open in Blender as he works, so the
picture is made from what the game reads rather than from them.
"""

import math
import sys

import bpy
from mathutils import Vector

args = sys.argv[sys.argv.index("--") + 1:]
assert len(args) % 2 == 0, "pairs of a model and its picture"
for source, out in zip(args[0::2], args[1::2]):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=source)
    scene = bpy.context.scene
    bpy.context.view_layer.update()

    # Every corner of every mesh, in world space, drawn 18 parts in a
    # hundred further out from the middle of them all.
    depsgraph = bpy.context.evaluated_depsgraph_get()
    points = []
    for o in scene.objects:
        if o.type != "MESH":
            continue
        mesh = o.evaluated_get(depsgraph).to_mesh()
        points.extend(o.matrix_world @ v.co for v in mesh.vertices)
        o.evaluated_get(depsgraph).to_mesh_clear()
    lo = Vector([min(p[i] for p in points) for i in range(3)])
    hi = Vector([max(p[i] for p in points) for i in range(3)])
    middle = (lo + hi) / 2
    grown = [middle + (p - middle) * 1.18 for p in points]

    cam = bpy.data.objects.new("Thumbnail camera", bpy.data.cameras.new("Thumbnail camera"))
    scene.collection.objects.link(cam)
    cam.data.lens = 50
    cam.rotation_euler = (math.radians(60.3), 0, math.radians(32.7))
    # Fitted to the square the picture is, or it is fitted to the scene's
    # own frame, wider than tall, and comes out smaller.
    scene.render.resolution_x = scene.render.resolution_y = 256
    scene.render.resolution_percentage = 100
    bpy.context.view_layer.update()
    flat = [c for p in grown for c in p]
    cam.location = cam.camera_fit_coords(bpy.context.evaluated_depsgraph_get(), flat)[0]
    size = (hi - lo).length
    cam.data.clip_start = max(0.01, size * 0.001)
    cam.data.clip_end = max(cam.data.clip_end, size * 20)
    scene.camera = cam

    world = bpy.data.worlds.new("Thumbnail sky")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.75, 0.85, 1.0, 1)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.9
    scene.world = world

    engines = [e.identifier for e in bpy.types.RenderSettings.bl_rna.properties["engine"].enum_items]
    scene.render.engine = "BLENDER_EEVEE" if "BLENDER_EEVEE" in engines else "BLENDER_EEVEE_NEXT"
    try:
        scene.eevee.taa_render_samples = 32
    except Exception:
        pass
    scene.render.resolution_x = scene.render.resolution_y = 256
    scene.render.resolution_percentage = 100
    scene.render.film_transparent = True
    scene.view_settings.view_transform = "Standard"
    scene.render.image_settings.file_format = "PNG"
    scene.render.image_settings.color_mode = "RGBA"
    scene.render.filepath = out
    bpy.ops.render.render(write_still=True)
    print("PICTURED %s -> %s, %.2f x %.2f x %.2f m" % (source, out, hi.x - lo.x, hi.y - lo.y, hi.z - lo.z))
