# Maps: the islands are Blender models

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## The islands are Blender models

`assets/roundtownmap.glb` is the town (Hajun's, 2026-10-30: "i made a new map
in replace of circlemap"; `circlemap1.glb`, the old town, stays in the repo
and out of the APK), `assets/testhomemap.glb` your home, and
`assets/lobbymap.glb` House Builder's lobby. Each is a Blender model (`.blend`
beside it), not code; `island::Venue` names the files. Since 2026-09-27 your
home is the plain `homemap.glb` with a 3 m high room (a doorway facing where
you arrive) and a 6 m wall to your right, for trying the camera against
ceilings and walls; House Builder's plots are still the plain `homemap.glb`. `src/island.rs` reads
every triangle back out of each spawned scene, and that is what players stand
on and bump into — no bounds are kept by hand anywhere. Reshape the land,
re-export, and the collision follows.

- **All three are loaded at startup and all stand at the origin.** A game of
  House Builder adds eight more, `Venue::Plot(0..8)`, a copy of `homemap.glb`
  for each builder, and takes them away when it ends. What keeps them all apart
  is `island::Venue`, carried by everything that moves or collides — players
  and town objects alike. Bodies only collide with what shares their venue,
  and only the island you are on is drawn (`show_where_i_am`). Anything added
  to the world without a `Venue` is drawn everywhere and left out of collision
  altogether.
- **A shape is read once per model** (`island::Islands`), so the eight plots
  share one.
- **Everyone arrives on a ring round the middle**, seat by seat, facing in
  (`island::arrival`, through `roundtown_net::arrival`, which the server
  goes by too): in the town 31.5 m out (`TOWN_ARRIVAL_RADIUS`), on the path
  round its pond, and 6 m out on the grass of the home, the lobby and the
  plots. 6 m out in the new town is in the pond, 0.4 m deep, inside a rim a
  jump gets out over and the computer's players never would.
- **Someone modelled into a map is not land.** Skinned meshes are left out of
  the shape: they are drawn where their skeleton puts them, not where their
  transform does, so read as land they stood somewhere else entirely. (The
  House Builder read that way was an invisible wall 23 m out from the
  fountain, on the road to them.) Give such a person a collider in code, as
  `builder::settle_house_builder` does.
- **The sea is everywhere the model has nothing underfoot,** at `WATER_Y` — z = -0.5
  in Blender. The land's flat top is z = 0 (`LAND_TOP`); the lowest ground on
  any island, the home's sand, is z = -0.2 (the new town's roads are at 0, its
  paving 0.25 over them), so keep anything walkable above about -0.25 or it
  counts as wading. Each island has
  its own sheet of it drawn (`sky::sea`), hidden and shown along with the island.
- **Water inside a map is a Blender object with a material called `water`**
  (`island::WATER_MATERIAL`), like `Water_fountain_water` in the town's fountain
  basin. The game draws it with the sea's own material, whatever it looks like
  in Blender, and leaves it out of collision, so players stand on the bottom,
  not the surface. The fountain's is 0.45 m up, over a floor at 0.05 and under
  a rim at 0.55: from inside, the rim looks a hand's width over the water, but
  it is half a metre over your feet. Since 2026-10-08 (Hajun asked for water
  in the "second and third smaller circle" too) the tower's two bowls hold
  water as well, `Water_fountain_water_middle` at 0.915 m and
  `Water_fountain_water_top` at 1.44 m, each 1 cm under its brim, with the
  tower's own 32 corners so its edge lies on the bowl's wall. They were added
  to `circlemap1.blend` by script and exported with the exporter's defaults,
  which reproduce Hajun's export of the town byte for byte (checked first);
  nothing else in the file changed. In the new town the fountain stands in
  the middle of a pond 60 m across (`water`, 0.5 m up over a floor at 0.1,
  inside a rim 0.7 m high), and its bowls' water came over from the old
  town as `water.001`, as Blender names a copy: a material called as
  another is, with a dot and a number after, is that one
  (`island::is_material`), for water, road, paint and bulbs alike.
- **Fixed in `roundtownmap.blend` on 2026-10-30** (my doing, before it was
  played): `Road`, the road under all the town's paving, and
  `peoplepathoutsideoffountain`, the path round the pond, were inside out,
  their tops facing down, which the game falls through; their faces were
  turned round (no triangle moved, checked over 3,000 points). The road's
  paint (`Circle.001`-`.004`, `Plane`-`Plane.004`) had no material, which
  glTF draws white and wholly metallic, near black, and the buses' way round
  is read off: it was given `roadwhite`, the old town's (0.86 grey). The
  water tower's legs and the booth under it, on the island east of the town
  (`Cylinder.001`, `Cube.001`), had none either: they were given
  `default_grey`, Blender's own look (0.8 grey, rough 0.5), as they showed in
  Blender. A default export of Hajun's file reproduced their glb byte for
  byte first; the export after differs in those materials and those faces'
  winding alone. Saved without a `.blend1` of mine, so that theirs is theirs.
- **The town's second version, later on 2026-10-30**, is Hajun's
  `roundtownmapbackup.blend` (untracked), the file in which they keep their
  modifiers (Hajun: "i dont want to apply the modifers in that file"):
  roundabouts with 8 m of road round their islands rather than 6, white
  lines down the edges of the roads, 12 street lamps, 10 fewer trees (14),
  and its houses grey with blue or pink roofs. At their asking it was copied
  over `roundtownmap.blend` and every modifier applied there, each object
  checked triangle for triangle against what its modifiers made; the
  boolean cutters (`Road Cutters`) deleted, as Hajun deleted them from the
  first version, or a default export has them as walls; the edge lines down
  the straight roads (`Plane.005`), which had no material, given `roadwhite`
  in `roundtownmap.blend` only (the backup was open in Blender); and a
  linked node group nothing used dropped. A default export of it is the
  backup's export with Apply Modifiers and Visible Objects ticked, node for
  node, but for that material: 49,584 triangles (33,696 before), 20,160 of
  them the lamps. The fixes above came over in the backup.
- **The town's third version, that evening**, made the same way from the
  backup (Hajun saved at 18:14), and the same check passes: 20 street lamps,
  each an object of its own (`streetlam`, `.001` to `.019`): a pair either
  side of each of the four gaps in the ring's strip of grass and of each of
  its corners, 72 m out, and four 43 m out, inside the ring, in line with
  the straight roads; 4 trees
  (14); the strip of grass down the middle of the ring (`Circle`) 0.25 m
  high rather than 0.5, under a bus's underside, and the planters 0.5 rather
  than 0.75; and the road under the town (`Road`) and round the roundabouts'
  islands (`Circle.007`) made slabs 2 m deep by a Solidify, the sides of the
  town's edge going down into the sea. 58,736 triangles, 33,600 of them the
  lamps. Their top and bottom are cut into triangles each its own way, which
  found a flaw in how a vertical line was tried against a triangle: its
  slack was a share of the triangle, and half a millimetre outside the
  road's edge a line met its bottom and missed its top, and was taken for
  inside it (a bus test failed there). The slack is a distance since
  (`island::EDGE`, 0.1 mm). Edge lines `Plane.005` still have no material
  in the backup.
- **What the old town taught, for any town**: a part with no material loads
  near black; keep a lamp an object of its own (the old town's 23 came joined
  three to an object, and were split by script, while whatever hid you faded:
  a lamp in the way faded its whole object; since nothing in the town fades
  (2026-10-25), the new town's 12 are one object as Hajun made them); keep a
  sign's font simple
  (Calibri Bold was 1,500 triangles a letter, Blender's own a few hundred);
  and the exporter leaves modifiers off by default (the old town's outer
  road dashes came as one dash and a Geometry Nodes Array): apply them, or
  tick Apply Modifiers when exporting.
- **Normals must point out of the land** (Mesh > Normals > Recalculate Outside).
  Up and down are read from the winding, so a top turned inside out is fallen
  through, even though the double-sided material hides it on screen.
- Faces steeper than 50° are walls, not floor. Anything thinner than 0.25 m can
  be walked through. Players wait where they arrive until their island has
  loaded.
- **A body steps up anything under 0.5 m** (`LAND_STEP_UP` in `src/lib.rs`,
  0.495; it was 0.75 until 2026-09-30, when Hajun made it 0.5). Half a metre
  itself, and anything higher, is a jump: later that day the fountain's rim,
  exactly 0.5 m over the floor of its basin, could be stepped out of, and
  Hajun wanted a jump. It is also where the band a body is tested against
  walls starts, so what stands between it and the head is in the way.
  Nothing else walkable in any map is exactly 0.5 m up from its
  neighbour (checked: only the fountain tower's tiers and some furniture
  tops, like the bookshelf's rows). An AI cannot jump, so it never steps
  down anything it could not step back up (`strands`, and the walks it
  picks in `ai::dry_all_the_way`): it keeps out of the fountain's basin.
- **A body is the shape of the model** (since 2026-10-08; of the old model,
  `limbperson.glb`: `playercharacter.glb`, since 2026-11-01, is narrower,
  0.35 m across the legs and 0.45 the shoulders, and bumps as wide as
  before): its legs, 0.42 m
  across (`LEG_RADIUS`), up to its hips at 0.65 m, then rounded out to its
  body and arms, 0.9 m across (`PLAYER_RADIUS`), from 0.8 m up to its head
  (`resolve_player_solids`, through `Island::walls_round`, which looks down
  each square of the lattice once for all three widths). Until then it was
  the full width from half a metre up, and the fountain's middle bowl could
  not be stood on: the top bowl, at the thighs of anyone on it, shoved them
  off into the basin.
- **A body stands on whatever is under its legs** (`support`): the ground
  under its middle, unless the ground under its legs, round that, is higher
  by more than a slope makes it (`EDGE_RISE`, 0.15 m), and then it is
  standing on an edge. Until 2026-10-08 only the middle counted: coming down
  with the legs on an edge and the middle just past it, a body sank past
  the edge, half into it, and was shoved out sideways once it reached its
  knees (Hajun: it "glitch steps", "any time" onto an edge). Missing an
  edge by more than the legs, the wider body above them is still eased off
  it, 0.12 m at most in a frame. Standing on the ground under its middle,
  as it nearly always is, a body does not look under its legs at all.
- **A solid has to be a closed shape.** Bodies are tested by dropping a
  vertical line through every triangle (`Island::blocked`), and where the
  surfaces it meets do not add up, the space is taken for the inside of
  something. A part left open — a gable with no bottom over a doorway — fills
  the doorway. Model roofs, gables and posts as closed shells.
  On a plot, each piece down on it is counted on its own, and only at the
  heights it reaches (since 2026-10-03). Before that the whole plot was
  counted as one, and a piece that is not a closed shape (most furniture: a
  blanket that is only a sheet, the cafe chair) threw out the count for
  everything over or under it. A chair downstairs put invisible walls on
  the upper floor over it, and a bed upstairs filled the room under it: in
  Hajun's two floor house the upper floor was sticky to drag pieces over,
  with spots nothing could go and players stuck. The islands themselves are
  still counted whole, but for the walls the game makes again round holes,
  each counted on its own, on a plot and in the town, and the town's window
  panes (since 2026-10-30). Counted with the rest of its house, the wall
  under the Square House's front gable was open to walk through: the gable
  is inside out, its underside facing up at the very height of the top of
  the wall, and two surfaces facing the same way at one height are taken
  for one, met on the seam between two of its triangles. Every town house
  had one wall a body walked out through (Hajun: "make sure player cannot
  go through the transparent house walls"); a test keeps it so
  (`a_wall_counted_on_its_own_is_solid_under_a_gable_inside_out`), and
  walking eight ways out of a town house in the desktop build, only the
  doorway lets you out.
- Anything baked into a map is scenery: the tap menu cannot move it. Only
  objects from the town save below, and House Builder's pieces while you
  build, can be edited.
