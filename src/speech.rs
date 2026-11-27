//! A bubble over the head of whoever says something (since 2026-11-27;
//! Hajun: "i want it so that the quickchat not only put the text in the
//! global chat, but also as a small textbubble that appears above the
//! player's head that other people can see if they are nearby").
//!
//! Whatever is said in your room ([`Spoke`]: online everyone's, you among
//! them, as the server tells it; offline your own) comes up over the head
//! of whoever said it, over their name, for [`SAY_SECS`], and pops away;
//! something else they say takes its place. It is seen only by whoever is
//! within [`HEARD_WITHIN`] of the one who said it, and only on the island
//! they are on.
//!
//! **It is a thing in the world**, as every bubble is since Hajun turned
//! down ones drawn on the screen (`bubble`; "make it a in-world ui button
//! that just rotates according to camera viewing angle, similar to how the
//! text in the "player's land" sign is made"): a sheet with a picture on
//! it, drawn once ([`balloon`]): a rounded box in the file's colour with a
//! tail at the foot of it, and in it the words in the game's own fonts
//! (Inter, and Pretendard for Hangul), as the words on a plot's sign are
//! drawn (`land`); turned every frame to face the camera squarely, its
//! tail's point just over the name. Unlit, its own colour by day and by
//! night. How big, how high, for how long and how near are mine.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use roundtown_net::Member;
use swash::scale::{Render, ScaleContext, Source};
use swash::zeno::{Format, Vector};

use crate::island::Venue;
use crate::net::Online;
use crate::ui::ui;
use crate::{Player, ThirdPersonCamera};

/// How long a bubble stays up, in seconds, how long it takes to pop up and
/// to pop away.
const SAY_SECS: f32 = 5.0;
const POP_SECS: f32 = 0.15;
/// How near the one who said it whoever sees it has to be, in metres.
const HEARD_WITHIN: f32 = 30.0;
/// How far over the point a name stands on (`tags::OVER_HEAD`) the point of
/// the tail is, in metres, the bubble seen from [`FULL_AT`]: over the name,
/// which is about a third of a metre tall from there.
const OVER_THE_NAME: f32 = 0.45;
/// How far off the camera the bubble is seen at its own size, in metres:
/// nearer, it is made smaller by as much as it is nearer, so that it is as
/// big on the screen as from there, as a name over a head is
/// (`tags::FULL_AT`); further off, smaller on the screen as anything is.
const FULL_AT: f32 = 8.0;
/// How tall a letter is, its em, in metres, seen from [`FULL_AT`]: half as
/// big again as a name's there; how far in from the edge of the box the
/// words begin, in ems; how much the corners are rounded, as a share of a
/// line; how tall the tail is and how wide at its root, in ems; and how
/// many pixels of the picture go to a metre.
const EM: f32 = 0.36;
const INSIDE: (f32, f32) = (0.6, 0.35);
const ROUND: f32 = 0.5;
const TAIL: (f32, f32) = (0.45, 0.55);
const PIXELS_A_METRE: f32 = 280.0;
/// The widest a line of words is, in ems, before it goes on to the next;
/// and the most lines.
const LINE_WIDE: f32 = 9.0;
const MOST_LINES: usize = 4;
/// How many times as bright as its picture it is drawn, being unlit, to
/// come out its own colour (`bubble::GAIN`, `land::WORDS_GAIN`).
const GAIN: f32 = 2.5;

/// Someone has said something: who, by their `Member::id` (none: you,
/// offline), and in this phone's words.
#[derive(Message, Clone, Debug, PartialEq)]
pub(crate) struct Spoke {
    pub who: Option<u64>,
    pub text: String,
}

/// A bubble: whose body it is over, how long it has been up, and how tall
/// its sheet is, in metres.
#[derive(Component)]
struct Speech {
    body: Entity,
    age: f32,
    high: f32,
}

pub(crate) fn plugin(app: &mut App) {
    app.add_message::<Spoke>()
        .add_systems(Update, (speak, hang.after(crate::follow_camera)).chain());
}

/// Puts up a bubble over the head of whoever said something, in place of
/// what they said before.
#[allow(clippy::type_complexity)]
fn speak(
    mut spoken: MessageReader<Spoke>,
    online: Res<Online>,
    me: Query<(Entity, &Venue), With<Player>>,
    others: Query<(Entity, &Member, &Venue), Without<Player>>,
    speeches: Query<(Entity, &Speech)>,
    (mut images, mut meshes, mut materials): (ResMut<Assets<Image>>, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    mut commands: Commands,
) {
    for said in spoken.read() {
        let mine = online.me().map(|me| me.id);
        let body = match said.who {
            Some(who) if Some(who) != mine => others.iter().find(|(_, member, _)| member.id == who).map(|(body, _, &venue)| (body, venue)),
            _ => me.single().ok().map(|(body, &venue)| (body, venue)),
        };
        let Some((body, venue)) = body else {
            continue;
        };
        for (before, speech) in &speeches {
            if speech.body == body {
                commands.entity(before).despawn();
            }
        }
        let picture = balloon(&said.text);
        let size = picture.metres();
        if size.x <= 0.0 || size.y <= 0.0 {
            continue;
        }
        let paint = materials.add(StandardMaterial {
            base_color: Color::linear_rgba(GAIN, GAIN, GAIN, 1.0),
            base_color_texture: Some(images.add(picture.image())),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        commands.spawn((
            Speech {
                body,
                age: 0.0,
                high: size.y,
            },
            Name::new("Speech bubble"),
            venue,
            Mesh3d(meshes.add(Rectangle::from_size(size))),
            MeshMaterial3d(paint),
            Transform::from_scale(Vec3::ZERO),
            Visibility::Hidden,
            NotShadowCaster,
        ));
    }
}

/// How big a bubble is `age` seconds after it came up, as a share of
/// itself: popping up, whole, and popping away at the end.
fn popped(age: f32) -> f32 {
    let up = (age / POP_SECS).clamp(0.0, 1.0);
    let away = ((SAY_SECS - age) / POP_SECS).clamp(0.0, 1.0);
    let ease = |t: f32| 1.0 - (1.0 - t).powi(3);
    ease(up).min(ease(away))
}

/// Hangs each bubble over its body's head, turned to the camera, while it
/// is near enough to you; and takes it away once its time is up, or its
/// body has gone.
#[allow(clippy::type_complexity)]
fn hang(
    time: Res<Time>,
    // Where everything is this frame, as the names over heads go by
    // (`tags`): the camera has no parent, nor has a body, so each one's
    // transform is its global one, and has just been set.
    cameras: Query<&Transform, (With<ThirdPersonCamera>, Without<Speech>)>,
    me: Query<(&Transform, &Venue), (With<Player>, Without<Speech>)>,
    bodies: Query<(&Transform, &Venue, Option<&crate::rest::Resting>), Without<Speech>>,
    mut speeches: Query<(Entity, &mut Speech, &mut Transform, &mut Visibility)>,
    mut commands: Commands,
) {
    let (Ok(eye), Ok((you, &here))) = (cameras.single(), me.single()) else {
        return;
    };
    let turn = eye.rotation;
    let up = turn * Vec3::Y;
    for (entity, mut speech, mut place, mut shown) in &mut speeches {
        speech.age += time.delta_secs();
        let Ok((body, &venue, resting)) = bodies.get(speech.body) else {
            commands.entity(entity).despawn();
            continue;
        };
        if speech.age >= SAY_SECS {
            commands.entity(entity).despawn();
            continue;
        }
        // Over where the name stands on the head (lower, sat or lying), by
        // as much less as it is made smaller near the camera.
        let head = body.translation + Vec3::Y * (crate::tags::OVER_HEAD - crate::rest::lower(resting));
        let fit = (eye.translation.distance(head) / FULL_AT).min(1.0);
        let tip = head + Vec3::Y * (OVER_THE_NAME * fit);
        let near = venue == here && tip.distance(you.translation) <= HEARD_WITHIN;
        shown.set_if_neq(if near { Visibility::Inherited } else { Visibility::Hidden });
        if !near {
            continue;
        }
        let scale = popped(speech.age) * fit;
        // The sheet's middle is half its height over the tail's point, up
        // the screen; it faces the camera along its own +Z.
        let now = Transform {
            translation: tip + up * (speech.high * 0.5 * scale),
            rotation: turn,
            scale: Vec3::splat(scale.max(1e-4)),
        };
        if *place != now {
            *place = now;
        }
    }
}

/// A bubble's picture: how much of each pixel the box covers and how much
/// the words do, row by row from the top, and how many pixels it is across
/// and down.
struct Balloon {
    shape: Vec<u8>,
    words: Vec<u8>,
    wide: usize,
    high: usize,
}

impl Balloon {
    /// How big it is in the world, across and up, in metres.
    fn metres(&self) -> Vec2 {
        Vec2::new(self.wide as f32, self.high as f32) / PIXELS_A_METRE
    }

    /// As a picture: the box in the file's colour, the words over it in
    /// theirs, with every smaller copy of itself down to a pixel, so that
    /// from further off it does not sparkle.
    fn image(&self) -> Image {
        let colours = ui().colours.chat;
        let back = colours.bubble.to_srgba();
        let ink = colours.bubble_text.to_srgba();
        // Each pixel, straight (not premultiplied) and in sRGB, as a
        // picture's are.
        let mut level: Vec<[f32; 4]> = self
            .shape
            .iter()
            .zip(&self.words)
            .map(|(&shape, &word)| {
                let (shape, word) = (f32::from(shape) / 255.0 * back.alpha, f32::from(word) / 255.0 * ink.alpha);
                let alpha = word + shape * (1.0 - word);
                if alpha <= 0.0 {
                    return [0.0; 4];
                }
                let mix = |a: f32, b: f32| (b * word + a * shape * (1.0 - word)) / alpha;
                [mix(back.red, ink.red), mix(back.green, ink.green), mix(back.blue, ink.blue), alpha]
            })
            .collect();
        let (mut wide, mut high) = (self.wide, self.high);
        let mut data = Vec::with_capacity(wide * high * 4 * 4 / 3 + 16);
        let mut levels = 0;
        loop {
            data.extend(level.iter().flat_map(|pixel| pixel.map(|n| (n.clamp(0.0, 1.0) * 255.0).round() as u8)));
            levels += 1;
            if wide == 1 && high == 1 {
                break;
            }
            (level, wide, high) = halved(&level, wide, high);
        }
        let rest = data.split_off(self.wide * self.high * 4);
        let mut image = Image::new(
            Extent3d {
                width: self.wide as u32,
                height: self.high as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.texture_descriptor.mip_level_count = levels;
        if let Some(data) = image.data.as_mut() {
            data.extend(rest);
        }
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            mipmap_filter: ImageFilterMode::Linear,
            ..ImageSamplerDescriptor::linear()
        });
        image
    }
}

/// `level`, `wide` by `high`, at half the size each way (never less than a
/// pixel), each pixel the mean of those it stands for, weighed by how much
/// of each is there, so that the edges do not darken.
fn halved(level: &[[f32; 4]], wide: usize, high: usize) -> (Vec<[f32; 4]>, usize, usize) {
    let (half_wide, half_high) = ((wide / 2).max(1), (high / 2).max(1));
    let mut half = Vec::with_capacity(half_wide * half_high);
    for y in 0..half_high {
        for x in 0..half_wide {
            let (mut sum, mut alpha, mut count) = ([0.0f32; 3], 0.0f32, 0.0f32);
            for (from_x, from_y) in [(2 * x, 2 * y), (2 * x + 1, 2 * y), (2 * x, 2 * y + 1), (2 * x + 1, 2 * y + 1)] {
                if from_x < wide && from_y < high {
                    let pixel = level[from_y * wide + from_x];
                    for (channel, value) in sum.iter_mut().zip(pixel) {
                        *channel += value * pixel[3];
                    }
                    alpha += pixel[3];
                    count += 1.0;
                }
            }
            let mean = |channel: f32| if alpha > 0.0 { channel / alpha } else { 0.0 };
            half.push([mean(sum[0]), mean(sum[1]), mean(sum[2]), alpha / count.max(1.0)]);
        }
    }
    (half, half_wide, half_high)
}

/// The letters of `line` in the game's fonts, each from the first that has
/// it, with how far it carries the pen, in ems.
fn letters<'a>(line: &str, fonts: &'a [swash::FontRef<'a>]) -> Vec<(&'a swash::FontRef<'a>, swash::GlyphId, f32)> {
    line.chars()
        .filter_map(|letter| {
            fonts.iter().find_map(|font| {
                let glyph = font.charmap().map(letter);
                (glyph != 0).then(|| (font, glyph, font.glyph_metrics(&[]).scale(1.0).advance_width(glyph)))
            })
        })
        .collect()
}

/// `text` broken into lines no wider than [`LINE_WIDE`] ems where it can be,
/// at its spaces, and no more than [`MOST_LINES`] of them: a word too long
/// for a line has one of its own.
fn lines(text: &str, fonts: &[swash::FontRef]) -> Vec<String> {
    let wide = |words: &str| letters(words, fonts).iter().map(|(.., carries)| carries).sum::<f32>();
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if wide(&format!("{line} {word}")) <= LINE_WIDE => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines.truncate(MOST_LINES);
    lines
}

/// Draws a bubble saying `text`: its lines, each in the middle across, in a
/// box with rounded corners as big as they need with room round them, and a
/// tail down from the middle of its foot. Nothing to draw, nothing at all.
fn balloon(text: &str) -> Balloon {
    let nothing = Balloon {
        shape: Vec::new(),
        words: Vec::new(),
        wide: 0,
        high: 0,
    };
    let fonts: Vec<swash::FontRef> = crate::font::files()
        .iter()
        .filter_map(|file| swash::FontRef::from_index(file, 0))
        .collect();
    let Some(first) = fonts.first() else {
        return nothing;
    };
    let said = lines(text, &fonts);
    let laid: Vec<Vec<(&swash::FontRef, swash::GlyphId, f32)>> = said.iter().map(|line| letters(line, &fonts)).collect();
    let long = |line: &Vec<(&swash::FontRef, swash::GlyphId, f32)>| line.iter().map(|(.., carries)| carries).sum::<f32>();
    let longest = laid.iter().map(long).fold(0.0, f32::max);
    if longest <= 0.0 {
        return nothing;
    }
    let metrics = first.metrics(&[]).scale(1.0);
    let line_high = metrics.ascent + metrics.descent + metrics.leading;
    // An em, in pixels.
    let size = EM * PIXELS_A_METRE;
    let box_wide = (longest + 2.0 * INSIDE.0) * size;
    let box_high = (line_high * laid.len() as f32 + 2.0 * INSIDE.1) * size;
    let tail_high = TAIL.0 * size;
    let wide = box_wide.ceil() as usize + 2;
    let high = (box_high + tail_high).ceil() as usize + 2;
    // The shape, by how far each pixel's middle is from its edge.
    let corner = (ROUND * line_high * size).min(box_high * 0.5);
    let middle = Vec2::new(wide as f32 * 0.5, 1.0 + box_high * 0.5);
    let half = Vec2::new(box_wide * 0.5, box_high * 0.5);
    let tail_root = TAIL.1 * size * 0.5;
    let foot = 1.0 + box_high;
    let mut shape = vec![0u8; wide * high];
    for y in 0..high {
        for x in 0..wide {
            let at = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            // The box: Inigo Quilez's rounded box.
            let q = (at - middle).abs() - half + Vec2::splat(corner);
            let in_box = q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0) - corner;
            // The tail: a triangle down from the foot, narrowing to its
            // point, as how far in from its two sides each pixel is.
            let down = at.y - foot;
            let in_tail = if down <= tail_high {
                let reach = tail_root * (1.0 - down.max(0.0) / tail_high);
                (at.x - middle.x).abs() - reach
            } else {
                f32::INFINITY
            };
            let edge = in_box.min(in_tail.max(down - tail_high).max(-down - 2.0));
            shape[y * wide + x] = ((0.5 - edge).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
    // The words.
    let mut words = vec![0u8; wide * high];
    let mut context = ScaleContext::new();
    for (row, line) in laid.iter().enumerate() {
        let baseline = 1.0 + (INSIDE.1 + row as f32 * line_high + metrics.leading * 0.5 + metrics.ascent) * size;
        let mut pen = (wide as f32 - long(line) * size) * 0.5;
        for &(font, glyph, carries) in line {
            let mut scaler = context.builder(*font).size(size).hint(false).build();
            let drawn = Render::new(&[Source::Outline])
                .format(Format::Alpha)
                .offset(Vector::new(pen.fract(), baseline.fract()))
                .render(&mut scaler, glyph);
            if let Some(drawn) = drawn {
                let (left, up) = (pen.floor() as i32 + drawn.placement.left, baseline.floor() as i32 - drawn.placement.top);
                let across = (drawn.placement.width as usize).max(1);
                for (at, &alpha) in drawn.data.iter().enumerate() {
                    let (x, y) = (left + (at % across) as i32, up + (at / across) as i32);
                    if x >= 0 && y >= 0 && (x as usize) < wide && (y as usize) < high {
                        let pixel = &mut words[y as usize * wide + x as usize];
                        *pixel = (*pixel).max(alpha);
                    }
                }
            }
            pen += carries * size;
        }
    }
    Balloon { shape, words, wide, high }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bubble_is_as_big_as_its_words_with_a_tail_under_it() {
        let short = balloon("Hi!");
        let long = balloon("Give me money.");
        assert!(short.wide > 0 && short.high > 0);
        assert!(long.metres().x > short.metres().x, "longer words, a wider bubble");
        assert!((long.metres().y - short.metres().y).abs() < 0.01, "one line each, as tall");
        // A short line is most of a metre tall with its tail, seen from
        // where it is its own size, and about as wide.
        assert!((0.6..1.1).contains(&short.metres().y), "{:?}", short.metres());
        // The words are inside the box, and the middle of its foot has the
        // tail under it, its sides clear.
        let at = |balloon: &Balloon, x: usize, y: usize| balloon.shape[y * balloon.wide + x];
        let foot = short.high - 3;
        assert!(at(&short, short.wide / 2, foot) > 0, "the tail");
        assert_eq!(at(&short, 1, foot), 0, "beside the tail, nothing");
        assert!(short.words.iter().any(|&word| word > 200), "the words are drawn");
        for (index, &word) in short.words.iter().enumerate() {
            if word > 0 {
                assert!(short.shape[index] > 0, "a letter outside the box at {index}");
            }
        }
        // Nothing to say, nothing drawn.
        assert_eq!(balloon("").wide, 0);
        assert_eq!(balloon("   ").wide, 0);
    }

    #[test]
    fn a_long_line_goes_on_to_the_next_and_hangul_is_drawn() {
        let fonts: Vec<swash::FontRef> = crate::font::files()
            .iter()
            .filter_map(|file| swash::FontRef::from_index(file, 0))
            .collect();
        let lines = lines("one two three four five six seven eight nine ten eleven twelve", &fonts);
        assert!(lines.len() > 1 && lines.len() <= MOST_LINES, "{lines:?}");
        assert_eq!(super::lines("Hi!", &fonts), ["Hi!"]);
        assert!(balloon("안녕하세요").words.iter().any(|&word| word > 200));
    }

    #[test]
    fn it_pops_up_stays_and_pops_away() {
        assert_eq!(popped(0.0), 0.0);
        assert_eq!(popped(POP_SECS), 1.0);
        assert_eq!(popped(SAY_SECS * 0.5), 1.0);
        assert_eq!(popped(SAY_SECS), 0.0);
        assert!(popped(SAY_SECS - POP_SECS * 0.5) < 1.0);
    }
}
