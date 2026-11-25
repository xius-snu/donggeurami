"""Turns out the faces of a house's upper floors that face into the floor,
in its own .blend, and exports the .glb beside it.

    blender --background "<house>.blend" --python blender/turn_floors_out.py
    blender --background "<house>.blend" --python blender/turn_floors_out.py -- <folder>

In Hajun's Square Houses of three to ten floors, each upper floor
(`Floor_2`, `Floor_2.001`, ...) has a piece over the foot of the stairs, 2 m
by 1.25 m, whose underside faced up (found 2026-11-25). The game takes the
space under a face that looks up for the inside of something solid
(`island::buried`), so that nobody could stand at the foot of the stairs, nor
go upstairs. Of each such object this turns round every flat face on its
underside that looks up and every one on its top that looks down, and
nothing else: no vertex moves.

Given a folder, it exports the .glb there and leaves the .blend as it was,
to compare first. Run again, it does nothing.
"""

import os
import sys

import bmesh
import bpy

folder = sys.argv[sys.argv.index("--") + 1] if "--" in sys.argv else None

turned = 0
for thing in bpy.data.objects:
    if thing.type != "MESH" or not thing.name.startswith("Floor_2"):
        continue
    mesh = thing.data
    world = thing.matrix_world
    heights = [(world @ vertex.co).z for vertex in mesh.vertices]
    bottom, top = min(heights), max(heights)
    shape = bmesh.new()
    shape.from_mesh(mesh)
    wrong = []
    for face in shape.faces:
        way = (world.to_3x3() @ face.normal).normalized()
        if abs(way.z) < 0.99:
            continue
        height = sum((world @ vertex.co).z for vertex in face.verts) / len(face.verts)
        if (abs(height - bottom) < 1e-4 and way.z > 0.0) or (abs(height - top) < 1e-4 and way.z < 0.0):
            wrong.append(face)
    if wrong:
        bmesh.ops.reverse_faces(shape, faces=wrong)
        shape.to_mesh(mesh)
        mesh.update()
    shape.free()
    print("TURNED %s: %d faces" % (thing.name, len(wrong)))
    turned += len(wrong)

for thing in bpy.data.objects:
    if thing.modifiers:
        print("MODIFIERS %s: %s" % (thing.name, [modifier.type for modifier in thing.modifiers]))

blend = bpy.data.filepath
name = os.path.splitext(os.path.basename(blend))[0] + ".glb"
if folder:
    bpy.ops.export_scene.gltf(filepath=os.path.join(folder, name), export_format="GLB")
    print("EXPORTED", os.path.join(folder, name), "turning", turned)
elif turned:
    # No .blend1 left beside it (none is kept since 2026-11-25: .gitignore).
    bpy.context.preferences.filepaths.save_version = 0
    bpy.ops.wm.save_mainfile(filepath=blend)
    bpy.ops.export_scene.gltf(filepath=os.path.join(os.path.dirname(blend), name), export_format="GLB")
    print("SAVED", blend, "turning", turned)
else:
    print("NOTHING TO TURN", blend)
