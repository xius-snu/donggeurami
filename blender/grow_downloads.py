"""Makes downloaded models already brought in by `scale_downloads.py` some
times bigger again, in their own .blend files, and exports each .glb
beside it as that script did.

    blender --background --factory-startup --python blender/grow_downloads.py -- 1.2 "<model>.blend" ...

Each file's one mesh is made FACTOR times as big about the middle of the
world, which is its origin: so a thing made 2.5 times its download's size
and grown 1.2 is three times it, as if `scale_downloads.py` had been run at
3 in the first place, and what stood on its origin still does. Its object
stays at the origin, unturned, its scale 1; its material, its picture and
its name are untouched.

Written 2026-11-25, when Hajun said the 56 of `mybuilds/newones` were meant
to be three times as big, not two and a half ("i think i told u to scale
some of the models in mybuilds/newones by 2.5x, but i meant 3x"): the
Painting stood up and the Treadmill set on its origin since are grown as
they are now.
"""

import os
import sys

import bpy
from mathutils import Matrix

args = sys.argv[sys.argv.index("--") + 1:]
factor, blends = float(args[0]), args[1:]
assert blends, "no .blend files given"
for blend in blends:
    bpy.ops.wm.open_mainfile(filepath=os.path.abspath(blend))
    meshes = [thing for thing in bpy.data.objects if thing.type == "MESH"]
    assert len(meshes) == 1, (blend, [thing.name for thing in meshes])
    mesh = meshes[0]
    assert mesh.parent is None, (blend, mesh.parent)
    assert mesh.data.users == 1, (blend, mesh.data.users)
    identity = Matrix.Identity(4)
    off = max(abs(mesh.matrix_world[row][col] - identity[row][col]) for row in range(4) for col in range(4))
    assert off < 1e-6, (blend, mesh.matrix_world)
    mesh.data.transform(Matrix.Scale(factor, 4))
    mesh.data.update()
    low = [min(v.co[k] for v in mesh.data.vertices) for k in range(3)]
    high = [max(v.co[k] for v in mesh.data.vertices) for k in range(3)]
    bpy.ops.wm.save_mainfile(filepath=os.path.abspath(blend))
    glb = os.path.splitext(os.path.abspath(blend))[0] + ".glb"
    bpy.ops.export_scene.gltf(filepath=glb, export_format="GLB")
    print("GREW %-22s %.2f m wide, %.2f deep, %.2f high, lowest %.3f" % (
        os.path.basename(blend), high[0] - low[0], high[1] - low[1], high[2] - low[2], low[2]))
