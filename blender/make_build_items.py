"""Make every piece House Builder's shop sells: a .blend to edit, the .glb the
game loads and the .png the shop shows, all in assets/build/.

    blender --background --python blender/make_build_items.py -- [kind ...]

With no kinds, it makes them all. Each is built from boxes, cylinders and
convex hulls so that it can be remade from here; once you have reshaped one in
Blender, export it again from its own .blend rather than running this over it.

What the game expects of each (see CLAUDE.md, "Building your house"):

* Blender units are metres, Z is up, and the front faces -Y.
* A house (foundation) stands on its origin at ground level, centred on its
  footprint. Every wall is its own object named "Wall...", an axis-aligned box
  from the ground to the eaves, at least 0.25 m thick or players walk through
  it: the game cuts doors and windows into these, so nothing else may be named
  that. Each floor is an object named "Floor...", which is how the game tells
  the inside of a wall from the outside. The roof and the gables are drawn as
  they are.
* A door or window has its origin at the bottom middle of the hole it needs,
  in the middle of the wall's thickness, X along the wall, and its +Y side
  facing into the house. Its frame is every object named "Frame...", and the
  box round them is the hole the game cuts: nothing else sets its size. How
  far up the wall a window goes is the game's to say.
* A painting or clock hangs with its origin at the middle of its back, which
  goes against the wall, and faces -Y, out of it. The box round the whole of
  it is how much wall it takes up.
* Everything else stands on its origin at floor level.
"""
import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(os.path.dirname(HERE), "assets", "build")

# Everything below is drawn at the size of a real house and real furniture,
# which is the size that suits the world's players, 1.7 m to the top of their
# heads: a door a head taller than they are, a bed at their knees. It is made
# this many times bigger before it is saved. It was 2.5 for a while, to give
# the camera room behind you indoors, but next to the players that was a house
# for giants; building is seen through your own eyes now. Change it here, run
# this again, and the game follows: it reads every size off the models. Below
# about 0.85 the walls get thinner than the game's collision can hold.
SCALE = 1.0

# --------------------------------------------------------------- materials


def linear(c):
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def material(name, hex_rgb, alpha=1.0, rough=0.75):
    m = bpy.data.materials.get(name)
    if m:
        return m
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = next(n for n in m.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
    rgb = [linear(int(hex_rgb[i:i + 2], 16) / 255) for i in (0, 2, 4)]
    bsdf.inputs['Base Color'].default_value = (*rgb, 1.0)
    bsdf.inputs['Roughness'].default_value = rough
    if alpha < 1.0:
        bsdf.inputs['Alpha'].default_value = alpha
        m.surface_render_method = 'BLENDED'
    return m


def mats():
    return {
        'wall': material("plain_wall", "EFEBE4", rough=0.9),
        'roof': material("plain_roof", "C9C5BE", rough=0.9),
        'floor': material("plain_floor", "D9D4CB", rough=0.9),
        'wood': material("wood", "A8743F"),
        'dark_wood': material("dark_wood", "6B4426"),
        'frame': material("frame_white", "F7F7F2"),
        'glass': material("glass", "A9D8F0", alpha=0.35, rough=0.1),
        'white': material("white", "F4F4F0"),
        'blue': material("fabric_blue", "4A78C2"),
        'teal': material("fabric_teal", "3B9C9C"),
        'teal_light': material("fabric_teal_light", "6CC2BD"),
        'red': material("red", "C8443C"),
        'pink': material("pink", "EE82A8"),
        'yellow': material("yellow", "F5C542"),
        'green': material("leaf_green", "4F9E47"),
        'dark_green': material("dark_green", "2F6B35"),
        'terracotta': material("terracotta", "C0663A"),
        'soil': material("soil", "4A3426"),
        'steel': material("steel", "B8BEC4", rough=0.3),
        'black': material("black", "2B2B2E"),
        'grey': material("grey", "8C9096"),
        'gold': material("gold", "D9A93B", rough=0.4),
        'sky': material("canvas_sky", "8CC8EE"),
        'hill': material("canvas_hill", "6DBA5A"),
        'lamp': material("lamp_shade", "FBE7B0"),
        'rug': material("rug_red", "B5473F"),
        'rug_edge': material("rug_edge", "E9C46A"),
        'orange': material("fabric_orange", "E07B39"),
        'orange_light': material("fabric_orange_light", "F2A66B"),
        'ceramic': material("ceramic_blue", "3F6FB5", rough=0.3),
        'mirror': material("mirror", "D6ECF5", rough=0.05),
    }

# ------------------------------------------------------------------ shapes


def link(name, bm, mat):
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    mesh.materials.append(mat)
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(obj)
    return obj


def box(name, lo, hi, mat):
    """An axis-aligned box from its lowest to its highest corner."""
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        v.co = Vector([lo[i] if v.co[i] < 0 else hi[i] for i in range(3)])
    return link(name, bm, mat)


def hull(name, points, mat):
    """The convex hull of `points`: a closed solid, normals out."""
    bm = bmesh.new()
    verts = [bm.verts.new(Vector(p)) for p in points]
    result = bmesh.ops.convex_hull(bm, input=verts)
    # Points left inside the hull are not part of it.
    bmesh.ops.delete(bm, geom=result['geom_interior'], context='VERTS')
    return link(name, bm, mat)


def slab(name, top, thickness, mat):
    """A slab whose top face is the flat polygon `top`, `thickness` deep."""
    a, b, c = (Vector(p) for p in top[:3])
    normal = (b - a).cross(c - a).normalized()
    if normal.z < 0:
        normal = -normal
    return hull(name, [Vector(p) for p in top] + [Vector(p) - normal * thickness for p in top], mat)


def cylinder(name, bottom, radius, height, mat, segments=20, top_radius=None):
    bm = bmesh.new()
    bmesh.ops.create_cone(
        bm, cap_ends=True, segments=segments, radius1=radius,
        radius2=radius if top_radius is None else top_radius, depth=height,
    )
    for v in bm.verts:
        v.co += Vector((bottom[0], bottom[1], bottom[2] + height / 2))
    return link(name, bm, mat)


def disc_y(name, center, radius, depth, mat, segments=28, tall=1.0):
    """A cylinder lying along Y, like a clock face on a wall: `tall` times as
    high as it is wide."""
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=segments, radius1=radius, radius2=radius, depth=depth)
    for v in bm.verts:
        v.co = Vector((v.co.x, v.co.z, v.co.y * tall)) + Vector(center)
    return link(name, bm, mat)


def lathe(name, profile, mat, segments=24):
    """A closed solid turned about Z: `profile` is (height, radius) from the
    bottom up."""
    bm = bmesh.new()
    rings = []
    for z, r in profile:
        rings.append([bm.verts.new((r * math.cos(a), r * math.sin(a), z))
                      for a in (2 * math.pi * i / segments for i in range(segments))])
    for low, high in zip(rings, rings[1:]):
        for i in range(segments):
            j = (i + 1) % segments
            bm.faces.new((low[i], low[j], high[j], high[i]))
    bm.faces.new(list(reversed(rings[0])))
    bm.faces.new(rings[-1])
    return link(name, bm, mat)


def sphere(name, center, radius, mat, scale=(1, 1, 1)):
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=16, v_segments=10, radius=radius)
    for v in bm.verts:
        v.co = Vector((v.co.x * scale[0], v.co.y * scale[1], v.co.z * scale[2])) + Vector(center)
    return link(name, bm, mat)

# ------------------------------------------------------------- the houses

WALL = 0.3      # thick, before SCALE: the game's collision misses anything under 0.25 m
EAVES = 2.8     # high
PITCH = math.radians(35)
ROOF = 0.15     # thick
OVER = 0.35     # overhang


def walls(m, rects, height=EAVES):
    """Walls from (x0, y0, x1, y1) footprints, each its own box."""
    for i, (x0, y0, x1, y1) in enumerate(rects):
        box(f"Wall_{i + 1}", (x0, y0, 0.0), (x1, y1, height), m['wall'])


def floor(m, rects):
    for i, (x0, y0, x1, y1) in enumerate(rects):
        box(f"Floor_{i + 1}", (x0, y0, 0.0), (x1, y1, 0.1), m['floor'])


def gable_along_x(m, name, x0, x1, y0, y1, gable_ends=(True, True)):
    """A pitched roof over x0..x1 by y0..y1, its ridge along X, with the
    triangles under it at the ends it is open at."""
    half = (y1 - y0) / 2
    mid = (y0 + y1) / 2
    ridge = EAVES + half * math.tan(PITCH)
    drop = OVER * math.tan(PITCH)
    lo, hi = x0 - OVER, x1 + OVER
    for side, edge in (("S", y0 - OVER), ("N", y1 + OVER)):
        slab(f"{name}_{side}", [(lo, edge, EAVES - drop), (hi, edge, EAVES - drop),
                                (hi, mid, ridge), (lo, mid, ridge)], ROOF, m['roof'])
    for end, (a, b) in zip(gable_ends, ((x0, x0 + WALL), (x1 - WALL, x1))):
        if end:
            hull(f"{name}_gable_{a:+.1f}", [(a, y0, EAVES), (a, y1, EAVES), (a, mid, ridge - ROOF),
                                           (b, y0, EAVES), (b, y1, EAVES), (b, mid, ridge - ROOF)], m['wall'])


def gable_along_y(m, name, x0, x1, y0, y1, gable_ends=(True, True)):
    half = (x1 - x0) / 2
    mid = (x0 + x1) / 2
    ridge = EAVES + half * math.tan(PITCH)
    drop = OVER * math.tan(PITCH)
    lo, hi = y0 - OVER, y1 + OVER
    for side, edge in (("W", x0 - OVER), ("E", x1 + OVER)):
        slab(f"{name}_{side}", [(edge, lo, EAVES - drop), (edge, hi, EAVES - drop),
                                (mid, hi, ridge), (mid, lo, ridge)], ROOF, m['roof'])
    for end, (a, b) in zip(gable_ends, ((y0, y0 + WALL), (y1 - WALL, y1))):
        if end:
            hull(f"{name}_gable_{a:+.1f}", [(x0, a, EAVES), (x1, a, EAVES), (mid, a, ridge - ROOF),
                                           (x0, b, EAVES), (x1, b, EAVES), (mid, b, ridge - ROOF)], m['wall'])


def box_house(m, w, d):
    x, y = w / 2, d / 2
    walls(m, [(-x, -y, x, -y + WALL), (-x, y - WALL, x, y),
              (-x, -y + WALL, -x + WALL, y - WALL), (x - WALL, -y + WALL, x, y - WALL)])
    floor(m, [(-x + WALL, -y + WALL, x - WALL, y - WALL)])


def cottage(m):
    box_house(m, 6.0, 6.0)
    gable_along_x(m, "Roof", -3.0, 3.0, -3.0, 3.0)


def long_house(m):
    box_house(m, 9.0, 4.4)
    gable_along_x(m, "Roof", -4.5, 4.5, -2.2, 2.2)


def l_house(m):
    # A wing 8 m across the front, and one 4 m wide going back on the right.
    w = WALL
    walls(m, [(-4.0, -4.0, 4.0, -4.0 + w),      # front
              (4.0 - w, -4.0 + w, 4.0, 4.0),    # right, all the way back
              (0.0, 4.0 - w, 4.0 - w, 4.0),     # back of the right wing
              (0.0, 0.0, w, 4.0 - w),           # inside of the right wing
              (-4.0 + w, -w, w, 0.0),           # back of the front wing
              (-4.0, -4.0 + w, -4.0 + w, 0.0)])  # left
    floor(m, [(-4.0 + w, -4.0 + w, 4.0 - w, -w), (w, -w, 4.0 - w, 4.0 - w)])
    gable_along_x(m, "Roof_front", -4.0, 4.0, -4.0, 0.0)
    # Its near end runs into the front wing's roof, up to the ridge.
    gable_along_y(m, "Roof_back", 0.0, 4.0, -2.0, 4.0, gable_ends=(False, True))


def flat_house(m):
    x = 3.0
    walls(m, [(-x, -x, x, -x + WALL), (-x, x - WALL, x, x),
              (-x, -x + WALL, -x + WALL, x - WALL), (x - WALL, -x + WALL, x, x - WALL)], height=3.0)
    floor(m, [(-x + WALL, -x + WALL, x - WALL, x - WALL)])
    box("Roof", (-x - 0.25, -x - 0.25, 3.0), (x + 0.25, x + 0.25, 3.2), m['roof'])
    box("Roof_rim", (-x - 0.25, -x - 0.25, 3.2), (x + 0.25, x + 0.25, 3.35), m['wall'])


def pyramid_house(m):
    box_house(m, 6.0, 6.0)
    e = 3.0 + OVER
    eave = EAVES - OVER * math.tan(PITCH)
    apex = (0.0, 0.0, EAVES + 3.0 * math.tan(PITCH))
    corners = [(-e, -e, eave), (e, -e, eave), (e, e, eave), (-e, e, eave)]
    for i, side in enumerate("SENW"):
        a, b = corners[i], corners[(i + 1) % 4]
        slab(f"Roof_{side}", [a, b, apex], ROOF, m['roof'])


# ----------------------------------------------------- doors and windows

JAMB = 0.06
DEPTH = 0.38    # a little more than the wall, which is 0.3


def frame(m, w, h, bottom=True, jamb=JAMB, mat_key='frame'):
    x = w / 2
    d = DEPTH / 2
    box("Frame_left", (-x, -d, 0.0), (-x + jamb, d, h), m[mat_key])
    box("Frame_right", (x - jamb, -d, 0.0), (x, d, h), m[mat_key])
    box("Frame_top", (-x, -d, h - jamb), (x, d, h), m[mat_key])
    if bottom:
        box("Frame_bottom", (-x, -d, 0.0), (x, d, jamb), m[mat_key])


def open_leaf(m, name, hinge_x, width, height, towards):
    """A door leaf swung open into the house (+Y), hinged at `hinge_x` and
    reaching `towards` the middle of the doorway when shut."""
    t = 0.05
    x0 = hinge_x if towards > 0 else hinge_x - t
    box(name, (x0, 0.02, 0.02), (x0 + t, 0.02 + width, height), m['wood'])
    knob_x = x0 + (t + 0.04 if towards > 0 else -0.04)
    sphere(f"{name}_knob", (knob_x, 0.02 + width - 0.12, 1.0), 0.04, m['gold'])


def door(m):
    w, h = 1.4, 2.3
    frame(m, w, h, bottom=False)
    open_leaf(m, "Door", -w / 2 + JAMB, w - 2 * JAMB, h - JAMB - 0.02, 1)


def double_door(m):
    w, h = 2.0, 2.4
    frame(m, w, h, bottom=False)
    leaf = (w - 2 * JAMB) / 2
    open_leaf(m, "Door_left", -w / 2 + JAMB, leaf, h - JAMB - 0.02, 1)
    open_leaf(m, "Door_right", w / 2 - JAMB, leaf, h - JAMB - 0.02, -1)


def archway(m):
    frame(m, 1.6, 2.5, bottom=False, jamb=0.1, mat_key='wood')


def window(m, w, h, bars=1):
    frame(m, w, h)
    d = 0.015
    box("Glass", (-w / 2 + JAMB, -d, JAMB), (w / 2 - JAMB, d, h - JAMB), m['glass'])
    box("Bar_across", (-w / 2 + JAMB, -0.03, h / 2 - 0.025), (w / 2 - JAMB, 0.03, h / 2 + 0.025), m['frame'])
    for i in range(bars):
        x = -w / 2 + w * (i + 1) / (bars + 1)
        box(f"Bar_up_{i}", (x - 0.025, -0.03, JAMB), (x + 0.025, 0.03, h - JAMB), m['frame'])
    box("Sill", (-w / 2 - 0.05, -DEPTH / 2 - 0.08, -0.04), (w / 2 + 0.05, DEPTH / 2, 0.02), m['frame'])


def small_window(m):
    window(m, 1.2, 1.1)


def wide_window(m):
    window(m, 2.0, 1.1, bars=2)


def tall_window(m):
    # No taller: from a sill of 0.9 m, a higher hole would let you climb
    # through it.
    window(m, 0.9, 1.5)

# ------------------------------------------------------------- furniture


def bed(m):
    box("Base", (-0.75, -1.1, 0.0), (0.75, 1.1, 0.35), m['wood'])
    box("Mattress", (-0.7, -1.05, 0.35), (0.7, 1.05, 0.55), m['white'])
    box("Blanket", (-0.72, -1.07, 0.4), (0.72, 0.35, 0.58), m['blue'])
    box("Pillow", (-0.5, 0.55, 0.55), (0.5, 0.95, 0.68), m['white'])
    box("Headboard", (-0.75, 1.1, 0.0), (0.75, 1.18, 1.0), m['dark_wood'])


def chair(m):
    s = 0.23
    for i, (x, y) in enumerate(((-s, -s), (s, -s), (-s, s), (s, s))):
        box(f"Leg_{i}", (x - 0.025, y - 0.025, 0.0), (x + 0.025, y + 0.025, 0.44), m['wood'])
    box("Seat", (-s - 0.02, -s - 0.02, 0.44), (s + 0.02, s + 0.02, 0.5), m['wood'])
    box("Back", (-s - 0.02, s - 0.03, 0.5), (s + 0.02, s + 0.03, 0.95), m['dark_wood'])


def table(m):
    x, y = 0.65, 0.4
    for i, (a, b) in enumerate(((-x, -y), (x, -y), (-x, y), (x, y))):
        a0 = a + (0.05 if a < 0 else -0.12)
        b0 = b + (0.05 if b < 0 else -0.12)
        box(f"Leg_{i}", (a0, b0, 0.0), (a0 + 0.07, b0 + 0.07, 0.72), m['wood'])
    box("Top", (-x, -y, 0.72), (x, y, 0.78), m['wood'])


def sofa(m):
    box("Base", (-1.0, -0.45, 0.0), (1.0, 0.45, 0.4), m['teal'])
    box("Back", (-1.0, 0.25, 0.4), (1.0, 0.45, 0.85), m['teal'])
    box("Arm_left", (-1.0, -0.45, 0.4), (-0.82, 0.45, 0.62), m['teal'])
    box("Arm_right", (0.82, -0.45, 0.4), (1.0, 0.45, 0.62), m['teal'])
    box("Cushion_left", (-0.8, -0.42, 0.4), (-0.01, 0.24, 0.52), m['teal_light'])
    box("Cushion_right", (0.01, -0.42, 0.4), (0.8, 0.24, 0.52), m['teal_light'])


def kitchen(m):
    box("Cabinet", (-1.0, -0.3, 0.0), (1.0, 0.3, 0.86), m['white'])
    box("Top", (-1.02, -0.32, 0.86), (1.02, 0.32, 0.9), m['black'])
    box("Sink", (-0.8, -0.2, 0.86), (-0.25, 0.2, 0.905), m['steel'])
    box("Stove", (0.2, -0.24, 0.9), (0.8, 0.24, 0.91), m['grey'])
    for i, (x, y) in enumerate(((0.35, -0.11), (0.65, -0.11), (0.35, 0.11), (0.65, 0.11))):
        cylinder(f"Burner_{i}", (x, y, 0.91), 0.08, 0.01, m['black'])
    for i, x in enumerate((-0.5, 0.0, 0.5)):
        box(f"Gap_{i}", (x - 0.005, -0.305, 0.05), (x + 0.005, -0.3, 0.8), m['grey'])
        box(f"Handle_{i}", (x - 0.12, -0.33, 0.7), (x - 0.04, -0.3, 0.72), m['steel'])


def bookshelf(m):
    w, d, h = 1.0, 0.35, 1.8
    box("Side_left", (-w / 2, -d / 2, 0.0), (-w / 2 + 0.04, d / 2, h), m['wood'])
    box("Side_right", (w / 2 - 0.04, -d / 2, 0.0), (w / 2, d / 2, h), m['wood'])
    box("Back", (-w / 2, d / 2 - 0.02, 0.0), (w / 2, d / 2, h), m['dark_wood'])
    colours = ['red', 'blue', 'yellow', 'green', 'teal', 'pink']
    for i, z in enumerate((0.0, 0.45, 0.9, 1.35, h - 0.04)):
        box(f"Shelf_{i}", (-w / 2, -d / 2, z), (w / 2, d / 2, z + 0.04), m['wood'])
        if i < 4:
            x = -w / 2 + 0.07
            for j in range(7):
                thick = 0.06 + 0.02 * ((i + j) % 3)
                tall = 0.28 + 0.05 * ((i * 3 + j) % 3)
                box(f"Book_{i}_{j}", (x, -d / 2 + 0.05, z + 0.04), (x + thick, d / 2 - 0.04, z + 0.04 + tall),
                    m[colours[(i + j) % len(colours)]])
                x += thick + 0.01
                if x > w / 2 - 0.15:
                    break


def armchair(m):
    c = m['orange']
    box("Base", (-0.5, -0.45, 0.0), (0.5, 0.45, 0.4), c)
    box("Back", (-0.5, 0.25, 0.4), (0.5, 0.45, 0.9), c)
    box("Arm_left", (-0.5, -0.45, 0.4), (-0.34, 0.45, 0.62), c)
    box("Arm_right", (0.34, -0.45, 0.4), (0.5, 0.45, 0.62), c)
    box("Cushion", (-0.32, -0.42, 0.4), (0.32, 0.24, 0.52), m['orange_light'])


def wardrobe(m):
    w, d, h = 1.2, 0.6, 2.0
    box("Plinth", (-w / 2 + 0.03, -d / 2 + 0.03, 0.0), (w / 2 - 0.03, d / 2 - 0.03, 0.08), m['dark_wood'])
    box("Body", (-w / 2, -d / 2, 0.08), (w / 2, d / 2, h), m['wood'])
    box("Top", (-w / 2 - 0.03, -d / 2 - 0.03, h), (w / 2 + 0.03, d / 2 + 0.03, h + 0.05), m['dark_wood'])
    box("Split", (-0.006, -d / 2 - 0.004, 0.12), (0.006, -d / 2, h - 0.04), m['dark_wood'])
    for i, x in enumerate((-0.09, 0.06)):
        box(f"Handle_{i}", (x, -d / 2 - 0.04, 0.95), (x + 0.03, -d / 2, 1.25), m['gold'])


def nightstand(m):
    w, d, h = 0.5, 0.4, 0.55
    box("Body", (-w / 2, -d / 2, 0.0), (w / 2, d / 2, h), m['wood'])
    box("Drawer", (-w / 2 + 0.04, -d / 2 - 0.012, 0.3), (w / 2 - 0.04, -d / 2, 0.49), m['dark_wood'])
    sphere("Knob", (0.0, -d / 2 - 0.03, 0.395), 0.025, m['gold'])
    cylinder("Lamp_base", (0.09, 0.04, h), 0.05, 0.13, m['white'])
    cylinder("Lamp_shade", (0.09, 0.04, h + 0.13), 0.11, 0.13, m['lamp'], top_radius=0.07)

# ----------------------------------------------------------- decorations


def flower_pot(m):
    cylinder("Pot", (0, 0, 0), 0.13, 0.3, m['terracotta'], top_radius=0.18)
    cylinder("Soil", (0, 0, 0.26), 0.16, 0.03, m['soil'])
    for i, (x, y, h, c) in enumerate(((0.0, 0.0, 0.55, 'pink'), (0.08, 0.05, 0.47, 'yellow'),
                                      (-0.07, 0.06, 0.5, 'red'), (0.02, -0.08, 0.45, 'pink'))):
        cylinder(f"Stem_{i}", (x, y, 0.28), 0.012, h - 0.28, m['green'], segments=8)
        sphere(f"Flower_{i}", (x, y, h), 0.06, m[c])
    sphere("Leaves", (0.0, 0.0, 0.34), 0.13, m['green'], scale=(1, 1, 0.5))


def potted_plant(m):
    cylinder("Pot", (0, 0, 0), 0.18, 0.4, m['terracotta'], top_radius=0.24)
    cylinder("Trunk", (0, 0, 0.35), 0.04, 0.6, m['dark_wood'], segments=10)
    sphere("Bush", (0.0, 0.0, 1.15), 0.38, m['green'])
    sphere("Bush_side", (0.18, -0.08, 0.95), 0.24, m['dark_green'])


def painting(m):
    # Hung: its back at y = 0 against the wall, facing -Y.
    w, h = 1.0, 0.75
    box("Frame", (-w / 2, -0.05, -h / 2), (w / 2, 0.0, h / 2), m['gold'])
    box("Sky", (-w / 2 + 0.07, -0.06, -h / 2 + 0.07), (w / 2 - 0.07, -0.05, h / 2 - 0.07), m['sky'])
    hull("Hill", [(-w / 2 + 0.07, -0.065, -h / 2 + 0.07), (w / 2 - 0.07, -0.065, -h / 2 + 0.07),
                  (w / 2 - 0.07, -0.065, -0.05), (0.1, -0.065, 0.05), (-w / 2 + 0.07, -0.065, -0.1),
                  (-w / 2 + 0.07, -0.06, -h / 2 + 0.07), (w / 2 - 0.07, -0.06, -h / 2 + 0.07),
                  (w / 2 - 0.07, -0.06, -0.05), (0.1, -0.06, 0.05), (-w / 2 + 0.07, -0.06, -0.1)], m['hill'])
    disc_y("Sun", (0.25, -0.066, 0.15), 0.08, 0.004, m['yellow'])


def clock(m):
    disc_y("Rim", (0, -0.03, 0), 0.25, 0.06, m['black'])
    disc_y("Face", (0, -0.065, 0), 0.22, 0.01, m['white'])
    box("Hour", (-0.012, -0.075, 0.0), (0.012, -0.07, 0.12), m['black'])
    box("Minute", (0.0, -0.08, -0.012), (0.17, -0.075, 0.012), m['black'])


def rug(m):
    box("Edge", (-1.0, -0.7, 0.0), (1.0, 0.7, 0.02), m['rug_edge'])
    box("Middle", (-0.88, -0.58, 0.0), (0.88, 0.58, 0.025), m['rug'])


def floor_lamp(m):
    cylinder("Base", (0, 0, 0), 0.18, 0.04, m['black'])
    cylinder("Pole", (0, 0, 0.04), 0.02, 1.35, m['black'], segments=10)
    cylinder("Shade", (0, 0, 1.3), 0.25, 0.3, m['lamp'], top_radius=0.13)


def vase(m):
    lathe("Vase", [(0.0, 0.09), (0.04, 0.12), (0.18, 0.17), (0.32, 0.15),
                   (0.44, 0.08), (0.54, 0.06), (0.6, 0.085)], m['ceramic'])
    for i, (x, y, h, c) in enumerate(((0.0, 0.02, 0.95, 'yellow'), (0.05, -0.03, 0.85, 'red'),
                                      (-0.05, -0.02, 0.88, 'pink'))):
        cylinder(f"Stem_{i}", (x, y, 0.55), 0.01, h - 0.55, m['green'], segments=8)
        sphere(f"Flower_{i}", (x, y, h), 0.055, m[c])


def mirror(m):
    # Hung like the painting: its back at y = 0, facing -Y.
    disc_y("Frame", (0, -0.03, 0), 0.32, 0.06, m['gold'], tall=1.35)
    disc_y("Glass", (0, -0.065, 0), 0.27, 0.01, m['mirror'], tall=1.35)


def wall_shelf(m):
    # Hung like the painting: its back at y = 0, the shelf standing out along -Y.
    box("Board", (-0.45, -0.22, -0.02), (0.45, 0.0, 0.02), m['wood'])
    for i, x in enumerate((-0.33, 0.3)):
        hull(f"Bracket_{i}", [(x, 0.0, -0.02), (x + 0.03, 0.0, -0.02), (x, 0.0, -0.18), (x + 0.03, 0.0, -0.18),
                             (x, -0.16, -0.02), (x + 0.03, -0.16, -0.02)], m['dark_wood'])
    for i, (x, tall, c) in enumerate(((-0.4, 0.22, 'red'), (-0.34, 0.2, 'blue'), (-0.28, 0.23, 'teal'))):
        box(f"Book_{i}", (x, -0.18, 0.02), (x + 0.05, -0.03, 0.02 + tall), m[c])
    cylinder("Pot", (0.2, -0.11, 0.02), 0.06, 0.1, m['terracotta'], top_radius=0.075)
    sphere("Plant", (0.2, -0.11, 0.16), 0.08, m['green'])

# ------------------------------------------------------------ the catalogue

PIECES = {
    "cottage": cottage,
    "long_house": long_house,
    "l_house": l_house,
    "flat_house": flat_house,
    "pyramid_house": pyramid_house,
    "door": door,
    "double_door": double_door,
    "archway": archway,
    "window": small_window,
    "wide_window": wide_window,
    "tall_window": tall_window,
    "bed": bed,
    "chair": chair,
    "table": table,
    "sofa": sofa,
    "kitchen": kitchen,
    "bookshelf": bookshelf,
    "armchair": armchair,
    "wardrobe": wardrobe,
    "nightstand": nightstand,
    "flower_pot": flower_pot,
    "potted_plant": potted_plant,
    "painting": painting,
    "clock": clock,
    "rug": rug,
    "floor_lamp": floor_lamp,
    "vase": vase,
    "mirror": mirror,
    "wall_shelf": wall_shelf,
}

# ----------------------------------------------------------- the picture


def picture(path):
    """A 256 px picture of whatever is in the scene, from the front right and
    a little above, on a clear background, lit like the game: a sun and sky."""
    scene = bpy.context.scene
    objects = [o for o in scene.objects if o.type == 'MESH']
    corners = [o.matrix_world @ Vector(c) for o in objects for c in o.bound_box]
    lo = Vector([min(c[i] for c in corners) for i in range(3)])
    hi = Vector([max(c[i] for c in corners) for i in range(3)])
    middle = (lo + hi) / 2
    radius = (hi - lo).length / 2

    cam = bpy.data.objects.new("Thumbnail camera", bpy.data.cameras.new("Thumbnail camera"))
    scene.collection.objects.link(cam)
    cam.data.lens = 50
    way = Vector((0.9, -1.4, 0.95)).normalized()
    fov = cam.data.angle
    cam.location = middle + way * (radius / math.sin(fov / 2) * 1.02)
    cam.rotation_euler = (-way).to_track_quat('-Z', 'Y').to_euler()
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

    scene.render.engine = 'BLENDER_EEVEE'
    scene.eevee.taa_render_samples = 32
    scene.render.resolution_x = scene.render.resolution_y = 256
    scene.render.film_transparent = True
    scene.view_settings.view_transform = 'Standard'
    scene.render.image_settings.file_format = 'PNG'
    scene.render.image_settings.color_mode = 'RGBA'
    scene.render.filepath = path
    bpy.ops.render.render(write_still=True)


def make(kind):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    PIECES[kind](mats())
    # Bigger, about the origin, in the meshes themselves: a metre in Blender
    # is a metre in the game.
    for obj in bpy.context.scene.objects:
        if obj.type == 'MESH':
            obj.data.transform(Matrix.Scale(SCALE, 4))
            obj.data.update()
    blend = os.path.join(OUT, kind + ".blend")
    glb = os.path.join(OUT, kind + ".glb")
    png = os.path.join(OUT, kind + ".png")
    bpy.ops.export_scene.gltf(
        filepath=glb,
        export_format='GLB',
        use_selection=False,
        export_yup=True,
        export_apply=True,
        export_materials='EXPORT',
        export_draco_mesh_compression_enable=False,
        export_cameras=False,
        export_lights=False,
        export_normals=True,
        export_texcoords=True,
        export_skins=True,
        export_animations=True,
    )
    picture(png)
    bpy.ops.wm.save_as_mainfile(filepath=blend)
    print("MADE", kind)


def main():
    kinds = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
    os.makedirs(OUT, exist_ok=True)
    for kind in kinds or PIECES:
        make(kind)


main()
