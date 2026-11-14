# Character models and how they are moved

> Moved out of `CLAUDE.md` on 2026-11-14, word for word, to keep that file
> short: `CLAUDE.md` has the rules and how things stand now; this is the full
> account of what was built, why, in whose words, what was tried, and what was
> and was not checked. Dated entries are newest first. Where the text says
> "above" or "below", or names another section in quotes, that section may be
> in another file here: the table in `CLAUDE.md` says which.

## Models

Authored in Blender and exported by `blender/export_glb.py` into `assets/`. A
metre in Blender is a metre in the world: anything the wrong size gets resized in
Blender and re-exported rather than scaled at spawn time (`map::ObjectDef`).

- **Every body is Hajun's `playercharacter.glb`** (`lobby::MODEL`, since
  2026-11-01: "try using playercharacter in asset folder instead of both
  boy/girl model for the main player and all the AI's"): every boy, the
  AIs, the House Builder and everyone online, whatever look they wear, but
  the girls, who are their `playercharactergirl.glb` since that night
  (`lobby::GIRL_MODEL`: "now there is playercharacter girl as well. use
  those as the player choice in the beginning of the game character
  selection"; the same rig, body and rest, with hair to the waist and a
  skirt, 3,440 triangles), and the AIs, who wear the two and Hajun's two
  recolourings of each (`roundtown_net::BOY_LOOKS`, `GIRL_LOOKS`, since
  2026-11-02; above, "The first time, you sign up") (`lobby::LOOKS` maps
  each look to its model and any other to the boy's; `lobby::models()` is
  every model, each once, which the APK's test keeps in and the walk's test
  reads the limbs of). A Rigify human metarig, scaled to 0.616 in the file (the
  game draws it as the file has it), under a skinned skin-coloured body with
  shoes, a blue top and dark trousers, and a mouth and two eyes hung rigid
  on the head bone: about 4,150 triangles in seven materials, hair among them,
  1.64 m to the top
  of a round head 0.53 m across, its eyes 1.38 m up, facing Blender's -Y
  as every model does, resting in an A-pose with its upper arms 60 degrees
  under level. That is Hajun's fourth export of the day (23:17, with hair;
  22:53 dressed and bare-headed); the first
  was a bare 1.70 m figure of 1,468 triangles, flat-shaded, with the arms
  28 degrees under level, the second the same smooth-shaded. The boy,
  `limbperson.glb`, the girl, `limbpersongirl.glb`,
  and the AIs' `AIpersonred.glb`, `AIpersonryellow.glb` and
  `AIpersonrpurple.glb` stay in `assets/` and out of the APK
  (`assets-left-out.txt`; a test keeps whatever `LOOKS` names in it).
- **The walk and the punch turn limbs about the body's own axes, whatever
  way the rig's bones lie** (`WalkBone::turned`, `bind_limbs` in
  `src/lib.rs`, since 2026-11-01). Each limb is bound by its bone's name
  (`walk_bone_kind`: `thigh.L`, `shin.L`, `upper_arm.L` and their `.R`;
  `Bone.005.L.001` and the like until then) with how it lies at rest in the
  body's space, the turns of every node from the body down to it, the
  loader's half turn among them; a swing is a turn about the body's
  side-to-side axis and the arms hang about its front-to-back axis, each
  turned down from however the model rests it to 75 degrees under level
  (`ARM_HANG_BELOW`, `stance_from_rest`, worked out as the arm is bound; a
  fixed 0.8 rad for the first export's arms, 1.02 for the old models'
  straight out, until that evening), and each thigh turned in the same
  way to stand 3.5 degrees out from straight down (`LEG_SPLAY`; Hajun,
  that night: "the legs are too wide apart while standing/walking": the
  model's rest 7 degrees out, its ankles 0.27 m apart, 0.19 stood so; the
  shin goes in with its thigh), put on the bone through its frame.
  Until then each turn was about an axis
  of the old rig's own bone space, and a new rig meant working them out
  again. Another rig needs only its bone names in `walk_bone_kind`; a test
  reads the model and checks every limb is found once and swings the right
  way (`every_limb_of_the_player_s_model_is_bound_and_swings_the_right_way`).
  Seen in the desktop build, scripted, from the front and the side: standing
  with the arms at the sides, walking with the legs striding, the back shin
  kicking up and the arms swinging against the legs, and the punch level in
  front. The sign-up's pictures (`assets/signup/boy.png`, `girl.png`) are
  rendered from the two models (`blender/render_person.py`, the same
  night). The colliders (`LEG_RADIUS`, `PLAYER_RADIUS`, "A
  body is the shape of the model") still bump as wide as the old model,
  0.42 m at the legs and 0.9 at the body, where this one is 0.35 and 0.45;
  what follows the model is the see-through's points on you
  (`see_through::RINGS`, `CROWN`), your eyes (`EYE_HEIGHT`, 1.38, "Your
  eyes are inside your head") and where the camera looks at a face while
  you talk (`talk::FACE`, 1.38). Names and the chat bubble still stand
  1.85 m up (`tags::OVER_HEAD`), 0.21 m over this head where they were
  0.14 over the old. Not yet on a phone.
