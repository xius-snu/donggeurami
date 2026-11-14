//! The game's fonts: Inter Regular for everything (Hajun, 2026-11-13: "I
//! want to change the font of most things to the font "Inter" Regular"), in
//! place of the one Bevy comes with, FiraMono, a fixed-width font cut down
//! to plain ASCII, which everything was written in until then; and, since
//! 2026-11-14, Pretendard Regular for Hangul, which Inter has none of
//! (Hajun: "i need korean text to be able to work in the global chat not
//! just english. same for username and send message").
//!
//! Nothing that draws words names a font: each takes Bevy's default
//! (`TextFont::default()`), which is whatever font asset sits under the
//! default handle. [`plugin`] puts Inter there as the app is built, before
//! anything is drawn, so every word on the screen is Inter's from the first
//! frame, the opening's cover included, with nothing loaded later to swap
//! in. The files are compiled in ([`INTER`], [`HANGUL`]), as
//! `assets/ui.ron` is: a phone needs nothing packaged for it (the APK
//! leaves the files themselves out).
//!
//! Inter is 4.1's static Regular (`extras/ttf/Inter-Regular.ttf` of
//! <https://github.com/rsms/inter/releases/tag/v4.1>), under the SIL Open
//! Font License, which is beside it (`assets/fonts/LICENSE-inter.txt`).
//! Latin, with every letter Vietnamese has, Greek and Cyrillic, and a good
//! many signs FiraMono's subset had not (an ellipsis, a middle dot, dashes,
//! arrows).
//!
//! **Hangul is Pretendard's** (1.3.9's static Regular,
//! `packages/pretendard/dist/public/static/alternative/Pretendard-Regular.ttf`
//! of <https://github.com/orioncactus/pretendard>, under the same licence,
//! `assets/fonts/LICENSE-pretendard.txt`; 2.7 MB, six times Inter): the
//! font Korean screens are mostly set in now, drawn to go with Inter, whose
//! Latin it is built on. It is not the game's font and draws no Latin
//! here: it is what a run of Hangul falls back on. Bevy's text is laid out
//! by parley, which asks its list of fonts (`FontCx`, fontique) for
//! another font whenever the one asked for has no shape for a letter, by
//! the letter's script; [`fall_back_on_hangul`] puts Pretendard in that
//! list and names it as what Hangul falls back on. So a Korean name over a
//! head, in the chat, on a plate, is Pretendard's, and the Latin beside it
//! Inter's as before, on one line: every screen as it was, to the pixel.
//! What the two draw between them is what may be typed
//! (`roundtown_net::text`), which a test here keeps so.
//!
//! Inter's digits are of different widths, a 1 narrower than an 8, as its
//! letters are; FiraMono's were all one width. Every count and clock on the
//! screen would shift a hair as its digits changed, the whole line with it
//! where it is centred, so every text asks for Inter's fixed-width digits,
//! its `tnum` feature ([`sized`]): the same shapes, each on the same width.

use std::sync::Arc;

use bevy::prelude::*;
use bevy::text::{FontCx, FontFeatureTag, FontFeatures, load_font_assets_into_font_collection};
use fontique::{Blob, FallbackKey, Script};

/// Inter Regular, as built into the game.
const INTER: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
/// Pretendard Regular, as built into the game: Hangul's letters.
const HANGUL: &[u8] = include_bytes!("../assets/fonts/Pretendard-Regular.ttf");
/// The family Pretendard calls itself, by which the list of fonts has it.
const HANGUL_FAMILY: &str = "Pretendard";
/// Hangul, as Unicode's scripts are named: what a run of Korean is laid
/// out as, syllables and letters alike.
const HANGUL_SCRIPT: Script = Script::from_bytes(*b"Hang");

/// Makes Inter the font everything is written in: the font under Bevy's
/// default handle, which until now was the one Bevy comes with. After
/// `DefaultPlugins`, whose text plugin puts its own there. And has Hangul
/// fall back on Pretendard, from the first frame.
pub(crate) fn plugin(app: &mut App) {
    let mut fonts = app.world_mut().resource_mut::<Assets<Font>>();
    if let Err(error) = fonts.insert(AssetId::default(), Font::from_bytes(INTER.to_vec())) {
        // Bevy's own font stays, and the game is as it was before Inter.
        warn!("could not make Inter the game's font: {error}");
    }
    let mut font_cx = app.world_mut().resource_mut::<FontCx>();
    if !fall_back_on_hangul(&mut font_cx) {
        warn!("could not make Pretendard the font Hangul is written in: Korean will not be drawn");
    }
    // Bevy clears its list of fonts whenever a font asset is taken away,
    // which nothing in the game does: should that ever happen, Pretendard
    // goes back in after it.
    app.add_systems(
        PostUpdate,
        keep_hangul.after(load_font_assets_into_font_collection).run_if(resource_changed::<Assets<Font>>),
    );
}

/// Puts Pretendard in the list of fonts the text is laid out with, if it
/// is not there, and names it as the font Hangul falls back on: whatever
/// font a text asks for, a letter of Hangul it has no shape for is drawn
/// in Pretendard's. Whether it is so now.
fn fall_back_on_hangul(font_cx: &mut FontCx) -> bool {
    let collection = &mut font_cx.collection;
    let family = match collection.family_id(HANGUL_FAMILY) {
        Some(family) => Some(family),
        // (Not copied: the list holds on to the bytes built into the game.)
        None => collection
            .register_fonts(Blob::new(Arc::new(HANGUL)), None)
            .first()
            .map(|(family, _)| *family),
    };
    let Some(family) = family else {
        return false;
    };
    let key = FallbackKey::new(HANGUL_SCRIPT, None);
    if collection.fallback_families(key).eq(std::iter::once(family)) {
        return true;
    }
    collection.set_fallbacks(key, std::iter::once(family))
}

/// Keeps Pretendard what Hangul falls back on as the font assets change.
fn keep_hangul(mut font_cx: ResMut<FontCx>) {
    fall_back_on_hangul(&mut font_cx);
}

/// The font of words `size_vmin` big, as a share of the short side of the
/// screen: the game's font, with its digits all one width, so that a
/// number counting does not shift what it stands in.
pub(crate) fn sized(size_vmin: f32) -> TextFont {
    TextFont {
        font_size: FontSize::VMin(size_vmin),
        font_features: fixed_digits(),
        ..default()
    }
}

/// Inter's fixed-width digits (`tnum`).
pub(crate) fn fixed_digits() -> FontFeatures {
    FontFeatures::from([FontFeatureTag::TABULAR_FIGURES])
}

#[cfg(test)]
mod tests {
    use fontique::{Collection, CollectionOptions};
    use roundtown_net::text;

    use super::*;

    /// Whether the TrueType file `font` has `text` in its name table
    /// (UTF-16BE, as a TrueType name is written).
    fn named(font: &[u8], text: &str) -> bool {
        let wide: Vec<u8> = text.encode_utf16().flat_map(u16::to_be_bytes).collect();
        font.windows(wide.len()).any(|window| window == wide)
    }

    #[test]
    fn the_fonts_built_in_are_inter_and_pretendard_and_are_left_out_of_the_apk_as_files() {
        // TrueType files, each with its own name in its name table.
        for (font, family) in [(INTER, "Inter"), (HANGUL, HANGUL_FAMILY)] {
            assert_eq!(&font[..4], &[0, 1, 0, 0], "{family}: a TrueType font");
            assert!(named(font, family), "{family}'s own name");
            assert!(named(font, "Regular"), "{family}: its style");
            assert!(named(font, "SIL Open Font License"), "{family}: its licence");
        }
        // Compiled in, so the APK need not carry the files: its list of
        // what it leaves out names each, and the licence beside it.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let list = std::fs::read_to_string(root.join("mobile/android/app/assets-left-out.txt"))
            .expect("the APK's list of what it leaves out");
        for name in ["Inter-Regular.ttf", "LICENSE-inter.txt", "Pretendard-Regular.ttf", "LICENSE-pretendard.txt"] {
            assert!(root.join("assets/fonts").join(name).is_file(), "assets/fonts/{name}");
            assert!(list.lines().any(|line| line.trim() == name), "{name} is not left out of the APK");
        }
        for licence in ["LICENSE-inter.txt", "LICENSE-pretendard.txt"] {
            let words = std::fs::read_to_string(root.join("assets/fonts").join(licence)).expect("a licence");
            assert!(words.contains("SIL OPEN FONT LICENSE Version 1.1"), "{licence}");
        }
    }

    #[test]
    fn hangul_falls_back_on_pretendard_whatever_font_a_text_asks_for() {
        let mut font_cx = FontCx::default();
        let key = FallbackKey::new(HANGUL_SCRIPT, None);
        assert_eq!(font_cx.collection.fallback_families(key).count(), 0, "nothing, to begin with");
        assert!(fall_back_on_hangul(&mut font_cx));
        let family = font_cx.collection.family_id(HANGUL_FAMILY).expect("Pretendard, in the list");
        assert_eq!(font_cx.collection.fallback_families(key).collect::<Vec<_>>(), vec![family]);
        // Asked for in Korean's own name too, as a text's language may
        // say.
        let korean = FallbackKey::from((HANGUL_SCRIPT, "ko"));
        assert_eq!(font_cx.collection.fallback_families(korean).collect::<Vec<_>>(), vec![family]);
        // Again is no more of it: once in the list, once what Hangul falls
        // back on.
        assert!(fall_back_on_hangul(&mut font_cx));
        assert_eq!(font_cx.collection.fallback_families(key).collect::<Vec<_>>(), vec![family]);
        assert_eq!(font_cx.collection.family_id(HANGUL_FAMILY), Some(family));
        // Cleared, as Bevy clears it when a font asset is taken away, it
        // goes back in.
        font_cx.collection.clear();
        assert!(font_cx.collection.family_id(HANGUL_FAMILY).is_none());
        assert!(fall_back_on_hangul(&mut font_cx));
        assert_eq!(font_cx.collection.fallback_families(key).count(), 1);
        // Nothing else falls back on it: Latin is Inter's.
        let latin = FallbackKey::new(Script::from_bytes(*b"Latn"), None);
        assert_eq!(font_cx.collection.fallback_families(latin).count(), 0);
    }

    #[test]
    fn every_letter_the_game_writes_is_drawn_by_inter_or_by_pretendard() {
        // Each font's own map from letters to shapes.
        let mut collection = Collection::new(CollectionOptions {
            shared: false,
            system_fonts: false,
        });
        let mut map_of = |font: &'static [u8]| {
            let families = collection.register_fonts(Blob::new(Arc::new(font)), None);
            let info = families.first().and_then(|(_, fonts)| fonts.first()).expect("a font").clone();
            info.charmap_index()
        };
        let (inter, hangul) = (map_of(INTER), map_of(HANGUL));
        let (inter, hangul) = (inter.charmap(INTER).expect("Inter's map"), hangul.charmap(HANGUL).expect("Pretendard's map"));
        let draws = |map: &fontique::Charmap, c: char| map.map(c).is_some_and(|glyph| glyph != 0);
        let (mut latin, mut korean) = (0, 0);
        for c in (0x20..0x11_0000).filter_map(char::from_u32).filter(|c| text::allowed(*c)) {
            // Hangul is what is laid out as Hangul, which falls back on
            // Pretendard; the rest is Inter's own.
            if text::width(c) == 2 {
                assert!(!draws(&inter, c), "{c:?}: Inter has none of Hangul");
                assert!(draws(&hangul, c), "{c:?} (U+{:04X}) is not in Pretendard", c as u32);
                korean += 1;
            } else {
                assert!(draws(&inter, c), "{c:?} (U+{:04X}) is not in Inter", c as u32);
                latin += 1;
            }
        }
        // Every syllable and the letters alone; an English keyboard, and
        // Latin-1's and Vietnamese's letters besides.
        assert_eq!(korean, 11_172 + 51);
        assert!(latin > 95 + 90 + 90, "{latin}");
        // The marks a letter may be typed in pieces with are Inter's too,
        // for as long as they are shown before they are put together.
        for mark in ['\u{300}', '\u{301}', '\u{303}', '\u{309}', '\u{323}'] {
            assert!(draws(&inter, mark), "U+{:04X}", mark as u32);
        }
    }

    #[test]
    fn every_text_asks_for_fixed_width_digits() {
        let font = sized(4.0);
        assert_eq!(font.font_size, FontSize::VMin(4.0));
        assert_eq!(font.font_features, fixed_digits());
        assert_ne!(font.font_features, FontFeatures::default());
        // Still the default font, which the plugin makes Inter.
        assert_eq!(font.font, TextFont::default().font);
    }
}
