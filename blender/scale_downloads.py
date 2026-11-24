"""Downloaded models made the game's size, each as a Blender file of its own
and a .glb exported from it.

    blender --background --factory-startup --python blender/scale_downloads.py -- 2.5 assets/mybuilds/resizeneeded

Every `<name> by <whoever> - <id>.glb` in the folder (as poly.pizza names
what it hands out) is brought in, taken out from under the empty the
download hangs it from (`RootNode`), made SCALE times as big about the
middle of the world, and has every transform applied, the download's own
hundredfold scale on its mesh with it: the one object is at the origin,
unturned, its scale 1, and a metre in the file is a metre in the game. It
is saved as `<name>.blend` and exported as it comes to `<name>.glb`, beside
the download, which is left where it is. The object keeps the name it came
with, its material and its picture, which is packed into the file.

Written 2026-11-24 for the 56 of CreativeTrio's that Hajun put in
`assets/mybuilds/resizeneeded` (docs/house-builder.md): one mesh and one
material each. A download with more than one mesh in it is refused, to be
looked at by hand.
"""

import glob
import os
import sys

import bpy
import mathutils

args = sys.argv[sys.argv.index("--") + 1:]
scale, folder = float(args[0]), args[1]
downloads = sorted(glob.glob(os.path.join(folder, "* by *.glb")))
assert downloads, "nothing called `<name> by <whoever>.glb` in %s" % folder
for source in downloads:
    name = os.path.basename(source).split(" by ")[0].strip()
    assert name, source
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=source)
    bpy.context.view_layer.update()
    objects = list(bpy.context.scene.objects)
    meshes = [o for o in objects if o.type == "MESH"]
    others = [o for o in objects if o.type != "MESH"]
    assert len(meshes) == 1, (name, [o.name for o in objects])
    # Where it is in the world, before it is let go of what it hangs from.
    grow = mathutils.Matrix.Scale(scale, 4)
    world = {o: o.matrix_world.copy() for o in meshes}
    for o in meshes:
        assert o.data.users == 1
        o.parent = None
        o.matrix_world = grow @ world[o]
    for o in others:
        assert o.type == "EMPTY", (name, o.name, o.type)
        bpy.data.objects.remove(o)
    bpy.context.view_layer.update()
    # Object > Apply > All Transforms.
    bpy.ops.object.select_all(action="DESELECT")
    for o in meshes:
        o.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.context.view_layer.update()
    for o in meshes:
        assert o.matrix_world == mathutils.Matrix.Identity(4), (name, o.matrix_world)
    # Its picture goes with the file.
    for image in bpy.data.images:
        if image.source == "FILE" and not image.packed_file:
            image.pack()
    mesh = meshes[0]
    low = [min(v.co[k] for v in mesh.data.vertices) for k in range(3)]
    high = [max(v.co[k] for v in mesh.data.vertices) for k in range(3)]
    triangles = sum(len(p.vertices) - 2 for p in mesh.data.polygons)
    bpy.ops.wm.save_as_mainfile(filepath=os.path.abspath(os.path.join(folder, name + ".blend")))
    bpy.ops.export_scene.gltf(filepath=os.path.abspath(os.path.join(folder, name + ".glb")), export_format="GLB")
    print("MADE %-22s %.2f m wide, %.2f deep, %.2f high, %d triangles (object %r)" % (
        name, high[0] - low[0], high[1] - low[1], high[2] - low[2], triangles, mesh.name))
