"""Lowers or raises a model's one mesh so that its lowest point is at its
origin's height, in its own .blend, and exports the .glb beside it as
`scale_downloads.py` did.

    blender --background "<model>.blend" --python blender/set_on_origin.py

Every piece stands on its origin at floor level (`make_build_items.py`); the
downloaded treadmill was 11 cm over it, and so stood in the air, its belt's
far end more than a step up (2026-11-25, when its belt came to carry whoever
stands on it, `docs/house-builder.md`). Run again, it does nothing.
"""

import os

import bpy
from mathutils import Matrix

meshes = [thing for thing in bpy.data.objects if thing.type == "MESH"]
assert len(meshes) == 1, [thing.name for thing in meshes]
mesh = meshes[0]
lowest = min((mesh.matrix_world @ vertex.co).z for vertex in mesh.data.vertices)
mesh.data.transform(Matrix.Translation((0.0, 0.0, -lowest)))
mesh.data.update()
print("LOWERED %.4f m" % lowest)
blend = bpy.data.filepath
bpy.ops.wm.save_mainfile(filepath=blend)
bpy.ops.export_scene.gltf(filepath=os.path.splitext(blend)[0] + ".glb", export_format="GLB")
print("SAVED", blend)
