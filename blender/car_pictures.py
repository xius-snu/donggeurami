"""The cars' pictures in the bag's shop, made from the pack's own previews.

    python blender/car_pictures.py            (from the repository's root)

Each of `assets/cars/previews/<name>.png` or `.webp` (the pack's, 1,260 by
1,000 or so, the car on a clear background) is cut to the car, made as big
as fits 230 px across, and set in the middle of a clear square 256 px a
side, as `assets/cars/<name>.png`: the size of every other picture in the
shop, which a card draws square (`bag::panel`, `PICTURE`). The previews
themselves are too big to load, a megabyte or more each once read, and
some are WebP, which the game does not read: they stay out of the APK
(`assets-left-out.txt`). The red sports car is the pack's `italia`, and
has its picture (`drive::CARS`).

Written 2026-11-25 (docs/vehicles.md). Needs Pillow.
"""

import glob
import os

from PIL import Image

PREVIEWS = "assets/cars/previews"
OUT = "assets/cars"
SIDE = 256
WIDEST = 230

made = 0
for path in sorted(glob.glob(os.path.join(PREVIEWS, "*"))):
    name, ext = os.path.splitext(os.path.basename(path))
    if ext.lower() not in (".png", ".webp"):
        continue
    preview = Image.open(path).convert("RGBA")
    # The car: whatever is not clear.
    solid = preview.getchannel("A").point(lambda alpha: 255 if alpha > 8 else 0)
    box = solid.getbbox()
    assert box, path
    car = preview.crop(box)
    scale = WIDEST / max(car.size)
    car = car.resize((round(car.width * scale), round(car.height * scale)), Image.LANCZOS)
    picture = Image.new("RGBA", (SIDE, SIDE), (0, 0, 0, 0))
    picture.alpha_composite(car, ((SIDE - car.width) // 2, (SIDE - car.height) // 2))
    out = os.path.join(OUT, name + ".png")
    picture.save(out, optimize=True)
    made += 1
    print("%-8s %s -> %s, the car %dx%d" % (name, os.path.basename(path), out, car.width, car.height))
print("%d pictures" % made)
