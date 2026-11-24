"""Stands the downloaded painting up to hang on a wall, in its own .blend.

    blender --background "assets/mybuilds/newones/Painting.blend" --python blender/stand_painting.py

It came lying down, its picture facing up (Blender's +Z), and the game hangs
a picture as `make_build_items.py` made the script's: its origin at the
middle of its back, which goes against the wall, facing -Y, out of it. A
quarter turn about X does that, its long side up and down; the turn is
applied, so that it is at rest, and the .blend saved and the .glb exported
beside it as `scale_downloads.py` exported them. Run once (2026-11-25, Hajun:
"painting is supposed to go on the wall, not the ground. rotate the blender
asset for that one and make it go on walls"); run again it would turn it
again. `FLIP=1` in the environment turns it upside down as well, about its
face, for a picture that comes out the wrong way up.
"""

import math
import os

import bpy
from mathutils import Matrix

meshes = [thing for thing in bpy.data.objects if thing.type == "MESH"]
assert len(meshes) == 1, [thing.name for thing in meshes]
painting = meshes[0]
turn = Matrix.Rotation(math.radians(90.0), 4, "X")
if os.environ.get("FLIP") == "1":
    turn = Matrix.Rotation(math.radians(180.0), 4, "Y") @ turn
painting.data.transform(turn)
painting.data.update()
corners = [painting.matrix_world @ vertex.co for vertex in painting.data.vertices]
low = [min(corner[axis] for corner in corners) for axis in range(3)]
high = [max(corner[axis] for corner in corners) for axis in range(3)]
print("STOOD x %.3f..%.3f  y %.3f..%.3f  z %.3f..%.3f" % (low[0], high[0], low[1], high[1], low[2], high[2]))
blend = bpy.data.filepath
bpy.ops.wm.save_mainfile(filepath=blend)
bpy.ops.export_scene.gltf(filepath=os.path.splitext(blend)[0] + ".glb", export_format="GLB")
print("SAVED", blend)
