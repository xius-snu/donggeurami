//! Everything the player has put in their town, and the one file it lives in.
//!
//! The rules here are chosen so that moving to a server changes where a save is
//! kept and nothing else:
//!
//! * A town is **one JSON document**. `town.json` on disk today, one row's
//!   worth of a `maps` table later, with no reshaping in between.
//! * A record names its [kind](ObjectDef::kind), never an asset path, so a
//!   model can be renamed or re-exported without touching anyone's town.
//! * Everything that is the same for every copy of a kind — the model, what it
//!   collides with, whether the player may pick it up — lives in [`CATALOGUE`]
//!   and is never written to the save. Movable and immovable things are the
//!   same kind of record; only their catalogue entry differs.

use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::editor::{Editable, RING_SCALE, Touch, Untappable};
use crate::island::Venue;
use crate::net::Online;
use crate::{Collider, ColliderShape, TREE_TRUNK_HEIGHT, TREE_TRUNK_RADIUS};

/// Bumped when the shape of the save changes. A build that opens a save from a
/// newer version leaves it strictly alone: the town loads from [`default_town`]
/// and writing is switched off, so an old client can never flatten a new save.
const SAVE_VERSION: u32 = 1;

const SAVE_FILE: &str = "town.json";

/// One kind of thing that can stand in the town.
///
/// There is deliberately no scale field, here or in the save: a metre in
/// Blender is a metre in the world, so anything the wrong size gets resized in
/// Blender and re-exported rather than scaled at spawn time.
pub(crate) struct ObjectDef {
    /// What the save calls this kind. Never change one that has shipped.
    pub kind: &'static str,
    /// Path under `assets/`, e.g. `"models/bench.glb"`.
    pub model: &'static str,
    /// What the player bumps into, or `None` for something walked through.
    pub solid: Option<ColliderShape>,
    /// What a tap is tested against. Usually looser than `solid`, so that
    /// tapping the canopy of a tree counts and not just its trunk.
    pub touch: ColliderShape,
    /// Whether the edit menu opens on it at all. Scenery that belongs to the
    /// map rather than to the player says `false`.
    pub movable: bool,
}

impl ObjectDef {
    /// Half the width of the footprint a tap is tested against, which is what
    /// the selection ring is sized from.
    pub fn touch_radius(&self) -> f32 {
        match self.touch {
            ColliderShape::Cylinder { radius, .. } => radius,
            ColliderShape::Aabb { half_extents } => half_extents.x.max(half_extents.z),
        }
    }
}

/// Every kind that can appear in a town. Adding an entry here is all it takes
/// for saves to be able to name it.
pub(crate) const CATALOGUE: &[ObjectDef] = &[ObjectDef {
    kind: "tree",
    model: "tree.glb",
    solid: Some(ColliderShape::Cylinder {
        radius: TREE_TRUNK_RADIUS,
        height: TREE_TRUNK_HEIGHT,
    }),
    // The model stands about 9 m tall with two canopy lobes reaching 4 m out
    // from the trunk, so the tap target is much wider than the trunk the player
    // walks into: tapping the leaves should select the tree.
    touch: ColliderShape::Cylinder {
        radius: 3.0,
        height: 9.4,
    },
    movable: true,
}];

pub(crate) fn definition(kind: &str) -> Option<&'static ObjectDef> {
    CATALOGUE.iter().find(|def| def.kind == kind)
}

/// One placed object, as the save file stores it.
///
/// Flat and primitive-only on purpose: this is the row a server would keep, so
/// it has to survive a JSON round trip unchanged. Anything added later gets
/// `#[serde(default)]` so that an older save still loads.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct PlacedObject {
    /// Unique within one town and stable for the object's whole life, so a
    /// server can address it without matching on position.
    pub id: u64,
    pub kind: String,
    pub pos: [f32; 3],
    pub yaw_deg: f32,
}

/// One player's town: the whole of what gets written, and the whole of what a
/// server would store for them.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct TownSave {
    pub version: u32,
    /// The next free [`PlacedObject::id`]. Kept in the document so that ids
    /// stay unique without having to scan every object first.
    pub next_id: u64,
    pub objects: Vec<PlacedObject>,
}

/// A spawned object, tying the entity back to its row in the save.
#[derive(Component)]
pub(crate) struct MapObject {
    pub id: u64,
    pub def: &'static ObjectDef,
}

/// Where the save lives, and whether the world has drifted from it.
#[derive(Resource, Default)]
pub(crate) struct Town {
    /// `None` once writing is switched off — no writable directory, or a save
    /// this build would lose part of if it wrote it back.
    path: Option<PathBuf>,
    next_id: u64,
    dirty: bool,
}

impl Town {
    /// Note that something moved, turned, or went in the bin.
    pub fn touch(&mut self) {
        self.dirty = true;
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Town>()
        .add_systems(Startup, load_town)
        .add_systems(Update, hands_off_online)
        .add_systems(Last, save_town);
}

/// The town a player starts with. This is also where starting scenery goes:
/// everything on the map, movable or not, is one of these records. There is
/// none yet, so a new town is the island and nothing else.
fn default_town() -> TownSave {
    TownSave {
        version: SAVE_VERSION,
        next_id: 1,
        objects: Vec::new(),
    }
}

fn load_town(mut commands: Commands, assets: Res<AssetServer>, mut town: ResMut<Town>) {
    let path = save_path();
    let (save, mut writable) = match path.as_deref().map(read_save) {
        Some(Stored::Town(save)) => (save, true),
        Some(Stored::TooNew) => (default_town(), false),
        None => {
            warn!("no writable data directory: this town will not be saved");
            (default_town(), false)
        }
    };

    let mut next_id = save.next_id;
    for record in &save.objects {
        next_id = next_id.max(record.id + 1);
        let Some(def) = definition(&record.kind) else {
            // A kind this build has never heard of can only have come from a
            // newer one. Spawning it is impossible, and dropping it on the next
            // write would lose the player's object, so stop writing instead.
            warn!(
                "town save holds an unknown kind {:?}: leaving it out, and not writing over the file",
                record.kind
            );
            writable = false;
            continue;
        };
        spawn_object(&mut commands, &assets, def, record);
    }

    town.next_id = next_id;
    town.path = if writable { path } else { None };
}

fn spawn_object(
    commands: &mut Commands,
    assets: &AssetServer,
    def: &'static ObjectDef,
    record: &PlacedObject,
) {
    let mut object = commands.spawn((
        MapObject {
            id: record.id,
            def,
        },
        WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(def.model))),
        Transform::from_translation(Vec3::from(record.pos))
            .with_rotation(Quat::from_rotation_y(record.yaw_deg.to_radians())),
        // Like the island itself, what stands on the town is only seen, and
        // only in the way, while you are there.
        Venue::Town,
        Visibility::Hidden,
    ));
    if let Some(shape) = def.solid {
        object.insert(Collider { shape });
    }
    if def.movable {
        object.insert(Editable {
            touch: Touch::of(def.touch),
            ring: def.touch_radius() * RING_SCALE,
            turns: true,
            sticky: false,
        });
    }
}

/// Online the town is everyone's, and nobody else would see what you put in
/// it or moved: until homes are where things go (MULTIPLAYER.md, phase 4),
/// its objects cannot be tapped while you are online.
fn hands_off_online(
    online: Res<Online>,
    objects: Query<(Entity, Has<Untappable>), With<MapObject>>,
    mut commands: Commands,
) {
    if !online.is_changed() {
        return;
    }
    for (object, untappable) in &objects {
        if online.is() && !untappable {
            commands.entity(object).insert(Untappable);
        } else if !online.is() && untappable {
            commands.entity(object).remove::<Untappable>();
        }
    }
}

fn save_town(mut town: ResMut<Town>, objects: Query<(&MapObject, &Transform)>) {
    if !town.dirty {
        return;
    }
    town.dirty = false;
    let Some(path) = town.path.clone() else {
        return;
    };

    let mut records: Vec<PlacedObject> = objects
        .iter()
        .map(|(object, transform)| PlacedObject {
            id: object.id,
            kind: object.def.kind.to_string(),
            pos: transform.translation.to_array(),
            yaw_deg: transform
                .rotation
                .to_euler(EulerRot::YXZ)
                .0
                .to_degrees()
                .rem_euclid(360.0),
        })
        .collect();
    // A stable order keeps the file readable and matches the order a database
    // would hand the rows back in.
    records.sort_unstable_by_key(|record| record.id);

    let save = TownSave {
        version: SAVE_VERSION,
        next_id: town.next_id,
        objects: records,
    };
    if let Err(error) = write_save(&path, &save) {
        warn!("could not save the town to {}: {error}", path.display());
    }
}

/// What `town.json` had to say when the app opened.
enum Stored {
    /// A save this build understands — or no readable save, in which case this
    /// carries [`default_town`] and the file is safe to write over.
    Town(TownSave),
    /// A save from a newer build. Nothing gets written back over it.
    TooNew,
}

fn read_save(path: &Path) -> Stored {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        // No file yet is an ordinary first run, not worth a log line.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Stored::Town(default_town());
        }
        Err(error) => {
            warn!("could not read {}: {error}", path.display());
            return Stored::Town(default_town());
        }
    };
    match serde_json::from_str::<TownSave>(&text) {
        Ok(save) if save.version > SAVE_VERSION => Stored::TooNew,
        Ok(save) => Stored::Town(save),
        Err(error) => {
            warn!(
                "{} is not a town save ({error}): starting from the default town",
                path.display()
            );
            Stored::Town(default_town())
        }
    }
}

/// Writes beside the save and renames over it, so that being killed halfway
/// through leaves the old town intact rather than half a file.
fn write_save(path: &Path, save: &TownSave) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string_pretty(save).map_err(std::io::Error::other)?;
    let partial = path.with_extension("json.new");
    fs::write(&partial, text)?;
    fs::rename(&partial, path)
}

fn save_path() -> Option<PathBuf> {
    data_file(SAVE_FILE)
}

/// Where this device keeps the file called `name`: the town save, and who the
/// device is online (`net`).
#[cfg(target_os = "android")]
pub(crate) fn data_file(name: &str) -> Option<PathBuf> {
    // The app's own internal storage: writable, private, and removed with the
    // app. `ANDROID_APP` is set before Bevy starts, so it is always here.
    Some(
        bevy::android::ANDROID_APP
            .get()?
            .internal_data_path()?
            .join(name),
    )
}

#[cfg(target_os = "ios")]
pub(crate) fn data_file(name: &str) -> Option<PathBuf> {
    Some(kept_in_library(&PathBuf::from(std::env::var_os("HOME")?), name))
}

/// Where an iPhone keeps the file called `name`, in the app's sandbox at
/// `home`: in its Library, which is the app's own and which iOS backs up, and
/// not in Documents, which the Files app shows since 1.0.2 for the log
/// (`log_file`): the device's secret is nobody's to see. 1.0.1 kept them in
/// Documents, and what it left there is moved the first time it is looked
/// for.
#[cfg_attr(not(target_os = "ios"), allow(dead_code))]
fn kept_in_library(home: &Path, name: &str) -> PathBuf {
    let path = home.join("Library").join("Application Support").join(name);
    let old = home.join("Documents").join(name);
    if !path.exists() && old.exists() {
        let moved = path
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|()| fs::rename(&old, &path));
        if let Err(error) = moved {
            warn!("could not move {} to {}: {error}", old.display(), path.display());
        }
    }
    path
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn data_file(name: &str) -> Option<PathBuf> {
    /// Kept out of the project folder so that a rebuild never wipes a town.
    const APP_DIR: &str = "donggeurami_town";

    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
            })
    }?;
    Some(base.join(APP_DIR).join(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_1_0_1_left_in_documents_moves_to_library() {
        let home = std::env::temp_dir().join(format!("roundtown-home-{}", std::process::id()));
        let documents = home.join("Documents");
        fs::create_dir_all(&documents).unwrap();
        fs::write(documents.join("device.id"), "secret").unwrap();
        let path = kept_in_library(&home, "device.id");
        assert_eq!(path, home.join("Library").join("Application Support").join("device.id"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "secret");
        assert!(!documents.join("device.id").exists());
        // Nothing to move: only where it goes.
        let town = kept_in_library(&home, "town.json");
        assert!(!town.exists());
        let _ = fs::remove_dir_all(&home);
    }
}
