"""Renders one of Hajun's people, as a picture for the sign-up's character
choice: the model as it stands in its .blend, from the front, under a sun and
a plain sky, on a clear background. Adds a camera and a light in memory and
never saves the .blend.

blender --background --factory-startup person.blend --python render_person.py -- out.png

Or from an exported .glb, when the colouring has no .blend of its own (Hajun's
recolourings of 2026-11-02 are exports alone): an empty scene, the model
imported into it, and the same camera, sun and sky.

blender --background --factory-startup --python render_person.py -- out.png person.glb
"""
import bpy, sys, math
from mathutils import Vector

args = sys.argv[sys.argv.index('--') + 1:]
out = args[0]
if len(args) > 1:
    bpy.ops.wm.read_homefile(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=args[1])
scene = bpy.context.scene

# Where the person is: the box round every mesh of theirs, in world space.
# The glTF importer brings a mesh of its own along, a shape for the bones
# ("Icosphere", 2 m across at the origin, which renders as nothing), so where
# any mesh hangs off a parent, only those that do count.
meshes = [ob for ob in scene.objects if ob.type == 'MESH']
if any(ob.parent for ob in meshes):
    meshes = [ob for ob in meshes if ob.parent]
lo = Vector((1e9, 1e9, 1e9))
hi = Vector((-1e9, -1e9, -1e9))
for ob in meshes:
    for corner in ob.bound_box:
        p = ob.matrix_world @ Vector(corner)
        lo = Vector(map(min, lo, p))
        hi = Vector(map(max, hi, p))
middle = (lo + hi) * 0.5
height = hi.z - lo.z
print('BOX', tuple(round(v, 2) for v in lo), tuple(round(v, 2) for v in hi))

# A camera in front (Blender's -Y is the front every model faces), level
# with the middle of the body, far enough back to take the whole person in.
cam_data = bpy.data.cameras.new('SignupCamera')
cam_data.lens = 60.0
cam_data.sensor_fit = 'VERTICAL'
cam = bpy.data.objects.new('SignupCamera', cam_data)
scene.collection.objects.link(cam)
# Half the camera's vertical view, and how far back it has to stand for the
# person, with a little room over and under, to fill it.
half_view = math.atan(cam_data.sensor_height / (2.0 * cam_data.lens))
distance = height * 0.62 / math.tan(half_view)
cam.location = Vector((middle.x, lo.y - distance, middle.z))
cam.rotation_euler = (math.radians(90.0), 0.0, 0.0)
scene.camera = cam

# A sun from the front and above, a little to the left, as the shop's
# pictures have.
sun_data = bpy.data.lights.new('SignupSun', 'SUN')
sun_data.energy = 3.0
sun = bpy.data.objects.new('SignupSun', sun_data)
scene.collection.objects.link(sun)
sun.rotation_euler = (math.radians(55.0), 0.0, math.radians(-25.0))

# An even sky to fill the shadows.
world = scene.world or bpy.data.worlds.new('SignupWorld')
scene.world = world
world.use_nodes = True
bg = world.node_tree.nodes.get('Background')
if bg is not None:
    bg.inputs[0].default_value = (0.75, 0.8, 0.9, 1.0)
    bg.inputs[1].default_value = 0.6

engines = [e.identifier for e in bpy.types.RenderSettings.bl_rna.properties['engine'].enum_items]
scene.render.engine = 'BLENDER_EEVEE' if 'BLENDER_EEVEE' in engines else 'BLENDER_EEVEE_NEXT'
scene.render.resolution_x = 512
scene.render.resolution_y = 768
scene.render.resolution_percentage = 100
scene.render.film_transparent = True
scene.render.image_settings.file_format = 'PNG'
scene.render.image_settings.color_mode = 'RGBA'
scene.render.filepath = out
bpy.ops.render.render(write_still=True)
print('RENDERED', out)
