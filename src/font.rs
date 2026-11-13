//! The game's font: Inter Regular (Hajun, 2026-11-13: "I want to change the
//! font of most things to the font "Inter" Regular"), in place of the one
//! Bevy comes with, FiraMono, a fixed-width font cut down to plain ASCII,
//! which everything was written in until then.
//!
//! Nothing that draws words names a font: each takes Bevy's default
//! (`TextFont::default()`), which is whatever font asset sits under the
//! default handle. [`plugin`] puts Inter there as the app is built, before
//! anything is drawn, so every word on the screen is Inter's from the first
//! frame, the opening's cover included, with nothing loaded later to swap
//! in. The file is compiled in ([`INTER`]), as `assets/ui.ron` is: a phone
//! needs nothing packaged for it (the APK leaves the file itself out).
//!
//! It is Inter 4.1's static Regular (`extras/ttf/Inter-Regular.ttf` of
//! <https://github.com/rsms/inter/releases/tag/v4.1>), under the SIL Open
//! Font License, which is beside it (`assets/fonts/LICENSE-inter.txt`).
//! Latin, Greek and Cyrillic, and a good many signs FiraMono's subset had
//! not (an ellipsis, a middle dot, dashes, arrows): **no Hangul**, so names
//! are plain ASCII still (`roundtown_net::profile`).
//!
//! Its digits are of different widths, a 1 narrower than an 8, as its
//! letters are; FiraMono's were all one width. Every count and clock on the
//! screen would shift a hair as its digits changed, the whole line with it
//! where it is centred, so every text asks for Inter's fixed-width digits,
//! its `tnum` feature ([`sized`]): the same shapes, each on the same width.

use bevy::prelude::*;
use bevy::text::{FontFeatureTag, FontFeatures};

/// Inter Regular, as built into the game.
const INTER: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");

/// Makes Inter the font everything is written in: the font under Bevy's
/// default handle, which until now was the one Bevy comes with. After
/// `DefaultPlugins`, whose text plugin puts its own there.
pub(crate) fn plugin(app: &mut App) {
    let mut fonts = app.world_mut().resource_mut::<Assets<Font>>();
    if let Err(error) = fonts.insert(AssetId::default(), Font::from_bytes(INTER.to_vec())) {
        // Bevy's own font stays, and the game is as it was before Inter.
        warn!("could not make Inter the game's font: {error}");
    }
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
    use super::*;

    #[test]
    fn the_font_built_in_is_inter_regular_and_is_left_out_of_the_apk_as_a_file() {
        // A TrueType file, with Inter's name in its name table (UTF-16BE,
        // as a TrueType name is written).
        assert_eq!(&INTER[..4], &[0, 1, 0, 0], "a TrueType font");
        let wide = |text: &str| -> Vec<u8> { text.encode_utf16().flat_map(u16::to_be_bytes).collect() };
        let has = |text: &str| INTER.windows(wide(text).len()).any(|window| window == wide(text));
        assert!(has("Inter"), "Inter's own name");
        assert!(has("Regular"), "its style");
        // Compiled in, so the APK need not carry the file: its list of what
        // it leaves out names it, and the licence beside it.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let list = std::fs::read_to_string(root.join("mobile/android/app/assets-left-out.txt"))
            .expect("the APK's list of what it leaves out");
        for name in ["Inter-Regular.ttf", "LICENSE-inter.txt"] {
            assert!(root.join("assets/fonts").join(name).is_file(), "assets/fonts/{name}");
            assert!(list.lines().any(|line| line.trim() == name), "{name} is not left out of the APK");
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
