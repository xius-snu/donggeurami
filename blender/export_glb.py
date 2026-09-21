"""Export the current selection to assets/models/<name>.glb for 동그라미타운.

Blender GUI:  Scripting tab -> Open -> this file -> Run Script (Alt+P).
              Select the objects you want, then run. Nothing selected exports
              everything in the EXPORT collection.
Command line: blender town_template.blend --background --python blender/export_glb.py -- [name]

The export settings below are the only ones the project supports; running this
instead of the File > Export dialog is what keeps them from drifting.
"""
import bpy, sys, os, re

TRIANGULATABLE = {'MESH', 'CURVE', 'SURFACE', 'FONT', 'META'}
# Shader nodes that have no glTF equivalent and are silently dropped on export.
PROCEDURAL = {
    'TEX_NOISE', 'TEX_VORONOI', 'TEX_MUSGRAVE', 'TEX_WAVE', 'TEX_MAGIC',
    'TEX_CHECKER', 'TEX_BRICK', 'TEX_GRADIENT', 'VALTORGB', 'BUMP',
    'MIX_RGB', 'MIX', 'MATH', 'VECT_MATH', 'LAYER_WEIGHT', 'FRESNEL',
}


def repo_root():
    here = bpy.data.filepath
    if here:
        d = os.path.dirname(here)
        while d and d != os.path.dirname(d):
            if os.path.exists(os.path.join(d, "Cargo.toml")):
                return d
            d = os.path.dirname(d)
    return os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))


def snake(name):
    name = re.sub(r'[^0-9A-Za-z]+', '_', name).strip('_').lower()
    return name or "model"


def lint(objects):
    """Catch, before exporting, the things that silently come out wrong."""
    errors, warnings = [], []
    for o in objects:
        if o.type not in TRIANGULATABLE:
            continue

        s = o.scale
        if any(abs(v - 1.0) > 1e-4 for v in s):
            warnings.append(
                f"{o.name}: object scale is {tuple(round(v, 3) for v in s)}, not 1.0. "
                f"It exports as a baked node scale you cannot see from Bevy"
                + ("; non-uniform scale also distorts lighting normals. " if len(set(round(v, 4) for v in s)) > 1 else ". ")
                + "Ctrl+A -> All Transforms.")
        if o.rotation_euler and any(abs(v) > 1e-4 for v in o.rotation_euler):
            warnings.append(f"{o.name}: unapplied rotation. Ctrl+A -> All Transforms.")

        # Where does the geometry sit relative to its own origin?
        if o.type == 'MESH' and o.data.vertices:
            lo = min(v.co.z for v in o.data.vertices)
            if abs(lo) > 0.01:
                warnings.append(
                    f"{o.name}: lowest vertex is {lo:+.3f} m from the origin. For a prop that "
                    f"sits on the ground you usually want 0.000 so Transform::from_xyz(x, 0.0, z) "
                    f"plants it; move the geometry, not the object.")

        if not o.data.materials or all(m is None for m in o.data.materials):
            warnings.append(f"{o.name}: no material -> imports as glTF default white.")

        for m in o.data.materials:
            if m is None or not m.use_nodes:
                continue
            nodes = m.node_tree.nodes
            if not any(n.type == 'BSDF_PRINCIPLED' for n in nodes):
                errors.append(
                    f"{o.name} / {m.name}: no Principled BSDF. Every other shader node exports "
                    f"with no PBR block at all and imports as plain white.")
            proc = sorted({n.type for n in nodes if n.type in PROCEDURAL})
            if proc:
                warnings.append(
                    f"{o.name} / {m.name}: procedural nodes {proc} do not export. Flat values and "
                    f"image textures do — bake to an image, or split into more materials.")

        # Vertex colours only export when the node tree actually reads them.
        if o.type == 'MESH' and len(o.data.color_attributes):
            used = any(n.type in ('VERTEX_COLOR', 'ATTRIBUTE')
                       for m in o.data.materials if m and m.use_nodes
                       for n in m.node_tree.nodes)
            if not used:
                warnings.append(
                    f"{o.name}: has a colour attribute but no Color Attribute node wired into "
                    f"Base Color, so the colours are dropped.")
    return errors, warnings


def main():
    argv = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []

    objects = [o for o in bpy.context.selected_objects if not o.hide_select]
    if not objects:
        coll = bpy.data.collections.get("EXPORT")
        if coll:
            objects = list(coll.all_objects)
    if not objects:
        raise SystemExit("Nothing to export: select some objects, or put them in an EXPORT collection.")

    errors, warnings = lint(objects)
    for w in warnings:
        print("  WARN  " + w)
    for e in errors:
        print("  ERROR " + e)
    if errors:
        raise SystemExit(f"\nRefusing to export: {len(errors)} problem(s) above would import wrong.")

    name = snake(argv[0]) if argv else snake(
        bpy.context.view_layer.objects.active.name
        if bpy.context.view_layer.objects.active in objects else objects[0].name)
    out_dir = os.path.join(repo_root(), "assets", "models")
    os.makedirs(out_dir, exist_ok=True)
    out = os.path.join(out_dir, name + ".glb")

    for o in bpy.context.selected_objects:
        o.select_set(False)
    for o in objects:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]

    bpy.ops.export_scene.gltf(
        filepath=out,
        export_format='GLB',          # one self-contained file; Bevy reads .glb directly
        use_selection=True,
        export_yup=True,              # Blender Z-up -> glTF Y-up. Never turn this off.
        export_apply=True,            # bake modifiers into the mesh
        export_materials='EXPORT',
        export_draco_mesh_compression_enable=False,  # Bevy cannot decode Draco
        export_cameras=False,         # the game owns the camera
        export_lights=False,          # the game owns the lighting
        export_normals=True,
        export_texcoords=True,
        export_skins=True,
        export_animations=True,
    )
    print(f"\nWROTE {out}")
    print(f'Bevy:  asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/{name}.glb"))')


main()
