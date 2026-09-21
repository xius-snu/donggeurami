"""Generates blender/town_template.blend + assets/models/scale_reference.glb.

Run once via:
  blender --background --factory-startup --python blender/_make_template.py -- <repo_root>
Everything it builds is reproducible, so the .blend can be regenerated at will.
"""
import bpy, sys, os, math
from mathutils import Color

ROOT = sys.argv[sys.argv.index('--') + 1]

# Map constants mirrored from src/lib.rs. Keep in sync if those change.
PLANE_SIZE = 80.0
PLANE_LIMIT = 38.0
PLAYER_HEIGHT = 1.6
PLAYER_RADIUS = 0.45

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene

# 1 Blender unit == 1 metre == 1 Bevy unit. The glTF exporter ignores
# scale_length, so leaving it at 1.0 is what keeps the ruler honest.
scene.unit_settings.system = 'METRIC'
scene.unit_settings.scale_length = 1.0
scene.unit_settings.length_unit = 'METERS'


def srgb_hex(code):
    """#RRGGBB as typed in Blender's colour picker -> linear RGBA node value."""
    code = code.lstrip('#')
    c = Color((int(code[0:2], 16) / 255, int(code[2:4], 16) / 255, int(code[4:6], 16) / 255))
    lin = c.from_srgb_to_scene_linear()
    return (lin.r, lin.g, lin.b, 1.0)


def make_material(name, hex_code, roughness=0.8, metallic=0.0):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = srgb_hex(hex_code)
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    m.diffuse_color = srgb_hex(hex_code)  # solid-shading viewport colour
    return m


def new_collection(name):
    c = bpy.data.collections.new(name)
    scene.collection.children.link(c)
    return c


def move_to(obj, coll):
    for c in list(obj.users_collection):
        c.objects.unlink(obj)
    coll.objects.link(obj)


REFERENCE = new_collection("REFERENCE")
EXPORT = new_collection("EXPORT")

ref_grey = make_material("REF_Grey", "#3C3C3C")
ref_blue = make_material("REF_PlayerBlue", "#3A7ADB")

# --- Ground: the 80x80 m plane the game spawns, and the +/-38 m walkable edge.
bpy.ops.mesh.primitive_plane_add(size=PLANE_SIZE, location=(0, 0, 0))
ground = bpy.context.object
ground.name = "REF_Ground_80m"
ground.data.materials.append(make_material("REF_Ground", "#5C8C57"))

bpy.ops.mesh.primitive_plane_add(size=PLANE_LIMIT * 2, location=(0, 0, 0.002))
limit = bpy.context.object
limit.name = "REF_WalkableLimit_38m"
limit.display_type = 'WIRE'

# --- Player-sized stand-in: the exact cylinder src/lib.rs spawns.
bpy.ops.mesh.primitive_cylinder_add(
    radius=PLAYER_RADIUS, depth=PLAYER_HEIGHT, location=(1.2, 0, PLAYER_HEIGHT / 2)
)
player = bpy.context.object
player.name = "REF_Player_1m6"
player.data.materials.append(ref_blue)

# --- Front marker: Blender's -Y is the front, and becomes Bevy's forward.
bpy.ops.mesh.primitive_cone_add(radius1=0.18, depth=0.5, location=(0, -1.2, 0.25),
                                rotation=(math.radians(-90), 0, 0))
front = bpy.context.object
front.name = "REF_Front_is_MinusY"
front.data.materials.append(make_material("REF_FrontRed", "#D23B3B"))

for obj in (ground, limit, player, front):
    move_to(obj, REFERENCE)
    obj.hide_select = True  # so "select all + export" never picks them up

# --- A starter object in EXPORT, already following every rule: 1 m cube whose
# origin sits at its base, so Transform::from_xyz(x, 0.0, z) plants it on the ground.
bpy.ops.mesh.primitive_cube_add(size=1.0, location=(0, 0, 0))
starter = bpy.context.object
starter.name = "MyObject"
for v in starter.data.vertices:
    v.co.z += 0.5
bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)
starter.data.materials.append(make_material("MyObject_Body", "#E0A030"))
move_to(starter, EXPORT)

# --- Viewport: metres-scale clipping + shading that shows material colour.
for area in [a for w in bpy.data.workspaces for s in w.screens
             for a in s.areas if a.type == 'VIEW_3D']:
    for space in area.spaces:
        if space.type == 'VIEW_3D':
            space.clip_start = 0.01
            space.clip_end = 1000.0
            space.shading.type = 'MATERIAL'
            space.overlay.grid_scale = 1.0

blend_path = os.path.join(ROOT, "blender", "town_template.blend")
bpy.ops.wm.save_as_mainfile(filepath=blend_path, compress=True)
print("WROTE", blend_path)
