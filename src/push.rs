//! Where this phone is pushed word of your house while the game is not open
//! on it (since 2026-11-27; Hajun: "add push notification for both android
//! and ios - whenever someone visits your plot ... it will send a push
//! notification to you"; the pushes themselves are the server's,
//! `server/src/push.rs`).
//!
//! A phone has a token from its maker's service, which is where a push to
//! it goes: Firebase Cloud Messaging's on Android, Apple's Push
//! Notification service's on an iPhone. The game has it said to the
//! server as it comes online, and whenever it is new
//! (`house::Ask::Push`). It is had:
//!
//! * **On Android** from `MainActivity`, which asks Firebase for it as the
//!   app opens and hands it here a byte at a time, by numbers alone, as
//!   everything goes between the two (`friends::TO_COPY`, `signup`):
//!   [`Java_town_donggeurami_MainActivity_pushTokenByte`] and the two round
//!   it. And the activity asks the phone whether the game may show word at
//!   all (Android 13's `POST_NOTIFICATIONS`) when the game says to
//!   ([`Java_town_donggeurami_MainActivity_pushWanted`]).
//! * **On an iPhone** from UIKit, which hands it to the app's delegate once
//!   the app has asked for it. winit gives UIKit no delegate, so one of the
//!   game's own is set ([`ios`]): it hears the token and nothing else. Asking
//!   for it also asks whether the game may show word at all.
//! * **On desktop** from `RT_PUSH_TOKEN=android:<token>` or `ios:<token>`,
//!   for trying the server's side out; nothing otherwise.
//!
//! **When the phone is asked** whether word may be shown ([`ask_when_ready`]):
//! the first time you stand in the town online after the app opens, the
//! sign-up done and nothing up over the screen (mine). Android asks again
//! at the next opening if it was turned down once, and never after a
//! second time; an iPhone asks once ever, and after that only its settings
//! can change it.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;
use roundtown_net::Ask;
use roundtown_net::house::{self, Ask as HouseAsk, Platform};

use crate::Player;
use crate::hud::DialogUp;
use crate::island::Venue;
use crate::net::Online;
use crate::signup::Signup;

/// This phone's token, from its maker's service, once there is one.
static TOKEN: Mutex<Option<(Platform, String)>> = Mutex::new(None);

/// Whether the game wants the phone asked, now, whether word may be shown:
/// for `MainActivity` to see and do on Android.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
static WANTED: AtomicBool = AtomicBool::new(false);

/// `token` is this phone's, from `platform`'s service.
fn found(platform: Platform, token: String) {
    if !house::token_ok(&token) {
        warn!("a push token that is no token: left alone");
        return;
    }
    if let Ok(mut held) = TOKEN.lock() {
        *held = Some((platform, token));
    }
}

/// This phone's token, if it has one yet.
fn token() -> Option<(Platform, String)> {
    TOKEN.lock().ok()?.clone()
}

pub(crate) fn plugin(app: &mut App) {
    // On desktop, one made up for trying the server out.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if let Ok(said) = std::env::var("RT_PUSH_TOKEN")
        && let Some((platform, token)) = said.trim().split_once(':')
    {
        let platform = if platform == "ios" { Platform::Ios } else { Platform::Android };
        found(platform, token.to_string());
    }
    app.add_systems(Update, (ask_when_ready, tell_the_server));
}

/// Asks the phone whether the game may show word of your house, once a
/// launch: the first time you stand in the town online, the sign-up done
/// and nothing up over the screen. On an iPhone that is also where its
/// token is asked for; on Android the activity has asked Firebase already.
fn ask_when_ready(
    online: Res<Online>,
    (signup, dialog): (Res<Signup>, Res<DialogUp>),
    me: Query<&Venue, With<Player>>,
    mut asked: Local<bool>,
) {
    if *asked || !online.is() || signup.up() || dialog.0 || !me.single().is_ok_and(|&venue| venue == Venue::Town) {
        return;
    }
    *asked = true;
    WANTED.store(true, Ordering::Relaxed);
    #[cfg(target_os = "ios")]
    ios::ask();
}

/// Tells the server this phone's token as it comes online, and again
/// should the token change.
fn tell_the_server(online: Res<Online>, mut told: Local<Option<(Platform, String)>>, mut asks: MessageWriter<Ask>) {
    if !online.is() {
        // Told afresh on the next connection.
        *told = None;
        return;
    }
    let Some(now) = token() else {
        return;
    };
    if told.as_ref() == Some(&now) {
        return;
    }
    asks.write(Ask::House(HouseAsk::Push {
        platform: now.0,
        token: now.1.clone(),
    }));
    *told = Some(now);
}

// ----------------------------------------------------------------- Android

/// The token being handed over by `MainActivity`, a byte at a time.
#[cfg(target_os = "android")]
static HANDED: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// `MainActivity.pushTokenBegin()`: a token is about to be handed over.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn Java_town_donggeurami_MainActivity_pushTokenBegin(_env: *mut std::ffi::c_void, _class: *mut std::ffi::c_void) {
    if let Ok(mut handed) = HANDED.lock() {
        handed.clear();
    }
}

/// `MainActivity.pushTokenByte(int)`: the next byte of it.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn Java_town_donggeurami_MainActivity_pushTokenByte(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    byte: i32,
) {
    if let Ok(mut handed) = HANDED.lock()
        && handed.len() < house::TOKEN_LONGEST
    {
        handed.push(byte as u8);
    }
}

/// `MainActivity.pushTokenEnd()`: that was all of it.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn Java_town_donggeurami_MainActivity_pushTokenEnd(_env: *mut std::ffi::c_void, _class: *mut std::ffi::c_void) {
    let Ok(mut handed) = HANDED.lock() else {
        return;
    };
    let bytes = std::mem::take(&mut *handed);
    match String::from_utf8(bytes) {
        Ok(token) => {
            info!("Firebase has given this phone its token");
            found(Platform::Android, token);
        }
        Err(_) => warn!("a push token that is not text: left alone"),
    }
}

/// `MainActivity.pushWanted()`: 1 if the game wants the phone asked, now,
/// whether word may be shown, once; 0 otherwise.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "C" fn Java_town_donggeurami_MainActivity_pushWanted(_env: *mut std::ffi::c_void, _class: *mut std::ffi::c_void) -> i32 {
    i32::from(WANTED.swap(false, Ordering::Relaxed))
}

// ------------------------------------------------------------------ iPhone

/// The iPhone's side: a delegate of the game's own for UIKit to hand the
/// token to, and asking for it. **Never run here**: an iPhone build is
/// only type-checked on Windows (`mobile/ios/check_on_windows.sh`), and
/// what this does is read off the bindings' own source and Apple's
/// documents. If it goes wrong, the phone's `log.txt` says so first.
#[cfg(target_os = "ios")]
mod ios {
    use bevy::log::{info, warn};
    use block2::RcBlock;
    use objc2::rc::Retained;
    use objc2::runtime::{Bool, NSObject, NSObjectProtocol, ProtocolObject};
    use objc2::{ClassType, DeclaredClass, declare_class, msg_send_id, mutability};
    use objc2_foundation::{MainThreadMarker, NSData, NSError};
    use objc2_ui_kit::{UIApplication, UIApplicationDelegate};
    use objc2_user_notifications::{UNAuthorizationOptions, UNUserNotificationCenter};
    use roundtown_net::house::Platform;

    declare_class!(
        /// What UIKit hands the token to: the app's delegate, which winit
        /// leaves it without, hearing that and nothing else.
        struct PushDelegate;

        // SAFETY: NSObject has no rules for what is made of it; the
        // delegate is the main thread's, as UIKit's own are, and has
        // nothing to drop.
        unsafe impl ClassType for PushDelegate {
            type Super = NSObject;
            type Mutability = mutability::MainThreadOnly;
            const NAME: &'static str = "RoundTownPushDelegate";
        }

        impl DeclaredClass for PushDelegate {
            type Ivars = ();
        }

        unsafe impl NSObjectProtocol for PushDelegate {}

        unsafe impl UIApplicationDelegate for PushDelegate {
            #[method(application:didRegisterForRemoteNotificationsWithDeviceToken:)]
            fn registered(&self, _application: &UIApplication, token: &NSData) {
                let hex: String = token.bytes().iter().map(|byte| format!("{byte:02x}")).collect();
                info!("Apple has given this phone its token");
                super::found(Platform::Ios, hex);
            }

            #[method(application:didFailToRegisterForRemoteNotificationsWithError:)]
            fn failed(&self, _application: &UIApplication, error: &NSError) {
                warn!("no push token from Apple: {}", error.localizedDescription());
            }
        }
    );

    /// Has UIKit hand the token to a delegate of the game's own, asks
    /// whether word may be shown, and asks for the token. On the main
    /// thread, where the game's systems run on an iPhone; anywhere else,
    /// nothing.
    pub(super) fn ask() {
        let Some(main) = MainThreadMarker::new() else {
            warn!("not on the main thread: no push token asked for");
            return;
        };
        let application = UIApplication::sharedApplication(main);
        // SAFETY: a delegate of NSObject's, made as any is; the app holds
        // its delegate without keeping it, so it is never let go of.
        let delegate: Retained<PushDelegate> = unsafe { msg_send_id![super(main.alloc::<PushDelegate>().set_ivars(())), init] };
        // SAFETY: the main thread's, as the delegate is; and none was
        // there to put aside (winit sets none).
        unsafe {
            if application.delegate().is_none() {
                application.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            } else {
                warn!("the app has a delegate already: the push token may not come");
            }
        }
        std::mem::forget(delegate);
        let answered = RcBlock::new(|granted: Bool, _error: *mut NSError| {
            info!("word of your house may be shown: {}", granted.as_bool());
        });
        let options = UNAuthorizationOptions::UNAuthorizationOptionAlert
            | UNAuthorizationOptions::UNAuthorizationOptionSound
            | UNAuthorizationOptions::UNAuthorizationOptionBadge;
        // SAFETY: Apple's own, asked as its documents have it; the token
        // comes to the delegate above whatever is answered.
        unsafe {
            UNUserNotificationCenter::currentNotificationCenter().requestAuthorizationWithOptions_completionHandler(options, &answered);
            application.registerForRemoteNotifications();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_kept_only_if_it_is_one() {
        found(Platform::Android, "has space".into());
        assert_ne!(token().map(|(_, token)| token), Some("has space".into()));
        found(Platform::Android, "dGVzdA:APA91b".into());
        assert_eq!(token(), Some((Platform::Android, "dGVzdA:APA91b".into())));
    }
}
