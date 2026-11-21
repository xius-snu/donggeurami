"""Renders a picture of whatever is in a .blend, for its card in the bag's
window (the game's `bag::panel`): as the pieces of the shop are pictured
(`make_build_items.py`, `picture`), 256 px square, from the front right and
a little above, on a clear background, under a sun and a plain sky. Adds the
camera and the light in memory and never saves the .blend.

blender --background --factory-startup thing.blend --python render_thing.py -- out.png

Made for Hajun's car (`assets/car1.blend` to `assets/car1.png`, 2026-11-21),
which the bag has and House Builder's shop does not.
"""
import bpy, sys, math
from mathutils import Vector

out = sys.argv[sys.argv.index('--') + 1]
scene = bpy.context.scene

# The box round every mesh that is rendered, in world space.
objects = [o for o in scene.objects if o.type == 'MESH' and not o.hide_render]
corners = [o.matrix_world @ Vector(c) for o in objects for c in o.bound_box]
lo = Vector([min(c[i] for c in corners) for i in range(3)])
hi = Vector([max(c[i] for c in corners) for i in range(3)])
middle = (lo + hi) / 2
radius = (hi - lo).length / 2
print('BOX', tuple(round(v, 2) for v in lo), tuple(round(v, 2) for v in hi))

cam = bpy.data.objects.new("Thumbnail camera", bpy.data.cameras.new("Thumbnail camera"))
scene.collection.objects.link(cam)
cam.data.lens = 50
# From the front right (a model's front is Blender's -Y) and a little above.
way = Vector((0.9, -1.4, 0.95)).normalized()
fov = cam.data.angle
cam.location = middle + way * (radius / math.sin(fov / 2) * 1.02)
cam.rotation_euler = (-way).to_track_quat('-Z', 'Y').to_euler()
cam.data.clip_end = max(cam.data.clip_end, radius * 20)
scene.camera = cam

sun = bpy.data.objects.new("Thumbnail sun", bpy.data.lights.new("Thumbnail sun", 'SUN'))
scene.collection.objects.link(sun)
sun.data.energy = 3.5
sun.rotation_euler = (math.radians(45), 0, math.radians(35))

world = bpy.data.worlds.new("Thumbnail sky")
world.use_nodes = True
world.node_tree.nodes['Background'].inputs['Color'].default_value = (0.75, 0.85, 1.0, 1)
world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.9
scene.world = world

engines = [e.identifier for e in bpy.types.RenderSettings.bl_rna.properties['engine'].enum_items]
scene.render.engine = 'BLENDER_EEVEE' if 'BLENDER_EEVEE' in engines else 'BLENDER_EEVEE_NEXT'
try:
    scene.eevee.taa_render_samples = 32
except Exception:
    pass
scene.render.resolution_x = scene.render.resolution_y = 256
scene.render.resolution_percentage = 100
scene.render.film_transparent = True
scene.view_settings.view_transform = 'Standard'
scene.render.image_settings.file_format = 'PNG'
scene.render.image_settings.color_mode = 'RGBA'
scene.render.filepath = out
bpy.ops.render.render(write_still=True)
print('WROTE', out)
