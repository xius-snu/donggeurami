"""The cars of the pack in assets/cars/lowpoly_cars_premium, each made the
size of the car the game has and written as a .glb of its own.

    python blender/scale_pack_cars.py            (from the repository's root)
    python blender/scale_pack_cars.py 1.25 assets/cars assets/cars/lowpoly_cars_premium/GLTF/*.glb

The game's car, assets/car1.glb, is the pack's `italia` as Hajun brought it
into Blender: made 1.25 times as big about the middle of the world, every
transform applied, exported. This does the same to each of the pack's own
.glb files, in the file itself and not through Blender (whose importer makes
two materials of one where only some parts carry colours, and names a part
that has none `DefaultMaterial`):

- every part's own place is written into its points, and all of it is
  SCALE times as big about the middle of the world: every part's origin is
  the car's, as "Apply All Transforms" leaves it;
- a wheel is a part with "wheel" in its name (`car::is_wheel`): the two
  cars whose wheels the pack calls something else (tronco, yang) have them
  named `wheels.001` to `.004`;
- the colours some of the pack's meshes carry on their corners (COLOR_0,
  _1, _2) are left out: no material of the pack's uses them, the game would
  multiply the first into the picture, and on yang it is black over part of
  the body;
- where the game would not find four wheels, each wheel's loose pieces are
  put biggest first, the same triangles in another order (tronco, whose
  tyres' knobs are pieces of their own: the game takes a wheel's pieces in
  the order their triangles come, `car::split_wheels`, and counted 22).

Nothing else is touched: materials, pictures, normals, UVs and triangles are
the pack's to the byte. Needs numpy. Written 2026-11-24 (docs/vehicles.md).
"""

import glob
import json
import os
import struct
import sys

import numpy as np

SCALE = 1.25
PACK = "assets/cars/lowpoly_cars_premium/GLTF"
OUT = "assets/cars"
PAD = bytes(1)


def read_glb(path):
    data = open(path, "rb").read()
    magic, version, total = struct.unpack("<4sII", data[:12])
    assert magic == b"glTF" and version == 2 and total == len(data), path
    at = 12
    chunks = []
    while at < len(data):
        length, kind = struct.unpack("<I4s", data[at:at + 8])
        chunks.append((kind, data[at + 8:at + 8 + length]))
        at += 8 + length
    assert [kind for kind, _ in chunks] == [b"JSON", b"BIN" + PAD], path
    return json.loads(chunks[0][1]), chunks[1][1]


def write_glb(path, j, binary):
    text = json.dumps(j, separators=(",", ":")).encode("utf-8")
    text += b" " * (-len(text) % 4)
    binary = bytes(binary) + PAD * (-len(binary) % 4)
    total = 12 + 8 + len(text) + 8 + len(binary)
    with open(path, "wb") as out:
        out.write(struct.pack("<4sII", b"glTF", 2, total))
        out.write(struct.pack("<I4s", len(text), b"JSON"))
        out.write(text)
        out.write(struct.pack("<I4s", len(binary), b"BIN" + PAD))
        out.write(binary)


def loose_pieces(points, order):
    """As `car::loose_pieces`: triangles joined where corners are at one place
    (to a tenth of a millimetre), each piece in the order its first triangle
    comes, as its corners' numbers."""
    keys = [tuple(int(v) for v in np.round(p * 1e4)) for p in points]
    triangles = order.reshape(-1, 3)
    parent = list(range(len(triangles)))

    def root(at):
        while parent[at] != at:
            parent[at] = parent[parent[at]]
            at = parent[at]
        return at

    owner = {}
    for index, triangle in enumerate(triangles):
        for corner in triangle:
            other = owner.get(keys[corner])
            if other is None:
                owner[keys[corner]] = index
            else:
                a, b = root(other), root(index)
                parent[max(a, b)] = min(a, b)
    pieces = {}
    for index, triangle in enumerate(triangles):
        pieces.setdefault(root(index), []).extend(int(c) for c in triangle)
    return list(pieces.values())


def gather(boxes):
    """As `car::gather`: each piece to the first lot whose middle it is near."""
    lots = []
    for piece, (low, high) in enumerate(boxes):
        middle = (low + high) * 0.5
        near = None
        for lot in lots:
            size = max((lot[1] - lot[0]).max(), (high - low).max())
            if np.linalg.norm((lot[0] + lot[1]) * 0.5 - middle) < size * 0.5:
                near = lot
                break
        if near is None:
            lots.append([low.copy(), high.copy(), [piece]])
        else:
            near[0] = np.minimum(near[0], low)
            near[1] = np.maximum(near[1], high)
            near[2].append(piece)
    return lots


def is_wheel(name):
    name = (name or "").lower()
    return "wheel" in name and "steering" not in name


def wheels_as_the_game_counts(j, blobs):
    """The wheels the game will find, and how far apart its axles are."""
    accessors = j["accessors"]
    boxes = []
    for node in j["nodes"]:
        if "mesh" not in node or not is_wheel(node.get("name")):
            continue
        for prim in j["meshes"][node["mesh"]]["primitives"]:
            points = np.frombuffer(blobs[accessors[prim["attributes"]["POSITION"]]["bufferView"]], dtype="<f4").reshape(-1, 3)
            acc = accessors[prim["indices"]]
            order = np.frombuffer(blobs[acc["bufferView"]], dtype={5123: "<u2", 5125: "<u4"}[acc["componentType"]]).astype(np.int64)
            for piece in loose_pieces(points, order):
                boxes.append((points[piece].min(axis=0).astype(np.float64), points[piece].max(axis=0).astype(np.float64)))
    lots = gather(boxes)
    along = [float((lot[0][2] + lot[1][2]) * 0.5) for lot in lots]
    return len(lots), (max(along) - min(along)) if along else 0.0


def convert(source, out_dir, scale):
    j, binary = read_glb(source)
    name = os.path.splitext(os.path.basename(source))[0]
    nodes = j["nodes"]
    assert len(j["buffers"]) == 1 and "uri" not in j["buffers"][0]
    assert not any(k in n for n in nodes for k in ("rotation", "scale", "matrix")), "only places are written in: this one is turned or sized"
    assert not any("sparse" in a for a in j["accessors"]) and not any("byteStride" in v for v in j["bufferViews"])
    # Where each part is in the world: its own place and those of what it hangs from.
    parent = {child: i for i, n in enumerate(nodes) for child in n.get("children", [])}

    def place(i):
        at = np.zeros(3)
        while i is not None:
            at += np.array(nodes[i].get("translation", [0, 0, 0]), dtype=np.float64)
            i = parent.get(i)
        return at

    roots = j["scenes"][j.get("scene", 0)]["nodes"]
    assert len(roots) == 1 and nodes[roots[0]].get("name") == name, (roots, name)
    # Its wheels by name.
    renamed = []
    kids = nodes[roots[0]].get("children", [])
    if not any(is_wheel(nodes[k].get("name")) for k in kids):
        assert len(kids) == 4, kids
        for number, k in enumerate(sorted(kids, key=lambda k: nodes[k]["name"]), 1):
            renamed.append(nodes[k]["name"])
            nodes[k]["name"] = "wheels.%03d" % number
    views = j["bufferViews"]
    accessors = j["accessors"]
    blobs = [bytes(binary[v.get("byteOffset", 0):v.get("byteOffset", 0) + v["byteLength"]]) for v in views]
    used_meshes = [n["mesh"] for n in nodes if "mesh" in n]
    assert len(set(used_meshes)) == len(used_meshes), "a mesh shared between two parts"
    alone = lambda view: [a["bufferView"] for a in accessors].count(view) == 1
    seen = set()
    low, high = np.full(3, np.inf), np.full(3, -np.inf)
    for i, n in enumerate(nodes):
        if "mesh" in n:
            at = place(i)
            for prim in j["meshes"][n["mesh"]]["primitives"]:
                assert prim.get("mode", 4) == 4 and "targets" not in prim
                index = prim["attributes"]["POSITION"]
                assert index not in seen, "one set of points used twice"
                seen.add(index)
                acc = accessors[index]
                assert acc["type"] == "VEC3" and acc["componentType"] == 5126 and not acc.get("normalized")
                view = acc["bufferView"]
                assert alone(view) and acc.get("byteOffset", 0) == 0
                points = np.frombuffer(blobs[view], dtype="<f4").reshape(-1, 3).astype(np.float64)
                assert len(points) == acc["count"]
                moved = ((points + at) * scale).astype("<f4")
                blobs[view] = moved.tobytes()
                acc["min"] = [float(v) for v in moved.min(axis=0)]
                acc["max"] = [float(v) for v in moved.max(axis=0)]
                low = np.minimum(low, moved.min(axis=0))
                high = np.maximum(high, moved.max(axis=0))
        n.pop("translation", None)
    # A wheel's loose pieces biggest first, where the game would not find four wheels.
    reordered = 0
    if wheels_as_the_game_counts(j, blobs)[0] != 4:
        done = set()
        for n in nodes:
            if "mesh" not in n or not is_wheel(n.get("name")):
                continue
            for prim in j["meshes"][n["mesh"]]["primitives"]:
                index = prim["indices"]
                if index in done:
                    continue
                done.add(index)
                acc = accessors[index]
                view = acc["bufferView"]
                assert alone(view) and acc.get("byteOffset", 0) == 0
                kind = {5123: "<u2", 5125: "<u4"}[acc["componentType"]]
                order = np.frombuffer(blobs[view], dtype=kind).astype(np.int64)
                points = np.frombuffer(blobs[accessors[prim["attributes"]["POSITION"]]["bufferView"]], dtype="<f4").reshape(-1, 3)
                size = lambda piece: -float((points[piece].max(axis=0) - points[piece].min(axis=0)).max())
                again = np.array([corner for piece in sorted(loose_pieces(points, order), key=size) for corner in piece], dtype=kind)
                assert sorted(again.reshape(-1, 3).tolist()) == sorted(order.reshape(-1, 3).tolist())
                if again.tobytes() != blobs[view]:
                    blobs[view] = again.tobytes()
                    reordered += 1
    count, base = wheels_as_the_game_counts(j, blobs)
    assert count == 4, "%s: the game would find %d wheels" % (name, count)
    # No colours on the corners.
    dropped = 0
    for mesh in j["meshes"]:
        for prim in mesh["primitives"]:
            for key in [k for k in prim["attributes"] if k.startswith("COLOR_")]:
                del prim["attributes"][key]
                dropped += 1
    # Only what is still read: accessors, then views, numbered again in the order they were in.
    used = sorted({i for mesh in j["meshes"] for prim in mesh["primitives"]
                   for i in list(prim["attributes"].values()) + ([prim["indices"]] if "indices" in prim else [])})
    number = {old: new for new, old in enumerate(used)}
    for mesh in j["meshes"]:
        for prim in mesh["primitives"]:
            prim["attributes"] = {key: number[i] for key, i in prim["attributes"].items()}
            if "indices" in prim:
                prim["indices"] = number[prim["indices"]]
    accessors = j["accessors"] = [accessors[i] for i in used]
    kept = sorted({a["bufferView"] for a in accessors} | {im["bufferView"] for im in j.get("images", []) if "bufferView" in im})
    number = {old: new for new, old in enumerate(kept)}
    for a in accessors:
        a["bufferView"] = number[a["bufferView"]]
    for im in j.get("images", []):
        if "bufferView" in im:
            im["bufferView"] = number[im["bufferView"]]
    views = j["bufferViews"] = [views[i] for i in kept]
    blobs = [blobs[i] for i in kept]
    # The file again, each view in turn, on a boundary of four bytes.
    out = bytearray()
    for view, blob in zip(views, blobs):
        out += PAD * (-len(out) % 4)
        view["byteOffset"] = len(out)
        assert view["byteLength"] == len(blob)
        out += blob
    out += PAD * (-len(out) % 4)
    j["buffers"][0]["byteLength"] = len(out)
    write_glb(os.path.join(out_dir, name + ".glb"), j, out)
    size = high - low
    triangles = sum(a["count"] // 3 for a in (accessors[p["indices"]] for m in j["meshes"] for p in m["primitives"]))
    notes = []
    if renamed:
        notes.append("wheels named, from %s" % ", ".join(renamed))
    if reordered:
        notes.append("wheels' pieces put biggest first")
    if dropped:
        notes.append("corner colours left out")
    print("%-8s %.2f m long, %.2f wide, %.2f high, %.2f between the axles, %5d triangles%s" % (
        name, size[2], size[0], size[1], base, triangles, ("  (" + "; ".join(notes) + ")") if notes else ""))


if __name__ == "__main__":
    args = sys.argv[1:]
    scale = float(args[0]) if args else SCALE
    out_dir = args[1] if len(args) > 1 else OUT
    sources = args[2:] if len(args) > 2 else sorted(glob.glob(os.path.join(PACK, "*.glb")))
    assert sources, "no cars in %s: run it from the repository's root" % PACK
    for source in sources:
        convert(source, out_dir, scale)
