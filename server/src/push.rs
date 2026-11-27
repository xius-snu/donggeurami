//! Pushes: word sent to someone's phone while the game is not open on it
//! (since 2026-11-27, for `house`: someone is visiting your house, someone
//! has rated it). To an Android phone by Google's Firebase Cloud Messaging,
//! to an iPhone by Apple's Push Notification service, each with a key of
//! the game's own that is kept on the VM and nowhere else (`SERVER.md`,
//! "Pushes").
//!
//! The server is built with no C compiler (`deploy.sh`), so it has no TLS
//! of its own to reach either service with: each push is a run of `curl`,
//! which the VM has (Ubuntu's, which speaks HTTP/2, as Apple's service asks
//! for), on a thread of its own and never the game's loop. What curl is
//! told, a key's bearer among it, goes to it on its standard input and
//! never on its command line, where anyone on the machine could read it.
//! The keys are signed with in plain Rust: ES256 for Apple (`p256`), RS256
//! for Google (`rsa`).
//!
//! Set up from the environment, as the rest of the server is (`main`):
//!
//! * `RT_APNS_KEY`: the path to Apple's key for the service,
//!   `AuthKey_XXXXXXXXXX.p8`; `RT_APNS_KEY_ID`, its ten-letter id;
//!   `RT_APNS_TEAM`, the team's; `RT_APNS_TOPIC`, the app's bundle id,
//!   `town.donggeurami.app` unless set; and `RT_APNS_SANDBOX=1` for an app
//!   Xcode put on a phone itself (TestFlight's and the store's go by the
//!   service's production side).
//! * `RT_FCM`: the path to the service account Firebase's console gives as
//!   a JSON file for the project.
//!
//! Without them, whatever would have been pushed is only logged. A token
//! a service says is no longer anyone's is handed back ([`Pusher::gone`]),
//! for `house` to forget.

use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use bevy::prelude::*;
use roundtown_net::house::Platform;
use serde_json::json;

/// The app's bundle id, which Apple's service is told a push is for.
const TOPIC: &str = "town.donggeurami.app";
/// How long a bearer for Apple's service is used before another is signed:
/// Apple takes one between twenty minutes and an hour old.
const APPLE_BEARER_FOR: Duration = Duration::from_secs(40 * 60);
/// How long before Google's access token runs out another is asked for.
const GOOGLE_EARLY: Duration = Duration::from_secs(5 * 60);
/// The longest a run of curl is given, in seconds.
const CURL_SECS: u32 = 20;
/// The channel on an Android phone that the game's pushes come under, which
/// its activity makes (`MainActivity`).
const ANDROID_CHANNEL: &str = "house";

/// Something to push: to whose phone, by its maker's service and its
/// token; what it says; and what kind of word it is, a later one of which
/// takes its place on the phone rather than lying under it ("visit",
/// "rated").
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Push {
    pub platform: Platform,
    pub token: String,
    pub body: String,
    pub kind: &'static str,
}

/// Where pushes go: to the thread that sends them, with whatever services
/// are set up; and the tokens that thread has found to be nobody's.
#[derive(Resource)]
pub(crate) struct Pusher {
    jobs: Option<Sender<Push>>,
    gone: Option<Mutex<Receiver<String>>>,
}

impl Pusher {
    /// Nowhere: every push only logged.
    pub(crate) fn nowhere() -> Self {
        Self { jobs: None, gone: None }
    }

    /// Sends `push` on its way, or logs it, with no service set up for it.
    pub(crate) fn send(&self, push: Push) {
        match &self.jobs {
            Some(jobs) => {
                let _ = jobs.send(push);
            }
            None => info!("a push to {} ({:?}), with nothing set up to send it: {}", short(&push.token), push.platform, push.body),
        }
    }

    /// The tokens found to be nobody's since this was last asked.
    pub(crate) fn gone(&self) -> Vec<String> {
        let Some(gone) = &self.gone else {
            return Vec::new();
        };
        let Ok(gone) = gone.lock() else {
            return Vec::new();
        };
        gone.try_iter().collect()
    }
}

/// The beginning of a token, enough to tell one from another in the log.
fn short(token: &str) -> &str {
    token.get(..8).unwrap_or(token)
}

pub(crate) fn plugin(app: &mut App) {
    let apple = Apple::from_env();
    let google = Google::from_env();
    let pusher = if apple.is_none() && google.is_none() {
        warn!("no RT_APNS_KEY nor RT_FCM: nothing is pushed to anyone's phone");
        Pusher::nowhere()
    } else {
        info!(
            "pushes go to {}{}",
            if apple.is_some() { "iPhones" } else { "no iPhone" },
            if google.is_some() { " and Android phones" } else { ", no Android phone" }
        );
        let (jobs, to_worker) = channel();
        let (found_gone, gone) = channel();
        std::thread::Builder::new()
            .name("pushes".into())
            .spawn(move || work(apple, google, &to_worker, &found_gone))
            .expect("a thread for pushes");
        Pusher {
            jobs: Some(jobs),
            gone: Some(Mutex::new(gone)),
        }
    };
    app.insert_resource(pusher);
}

/// What came of a push.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sent {
    Delivered,
    /// The token is nobody's any more: the app was deleted, say.
    Gone,
    Failed,
}

/// The pushes' thread: sends each as it comes, by its maker's service.
fn work(mut apple: Option<Apple>, mut google: Option<Google>, jobs: &Receiver<Push>, found_gone: &Sender<String>) {
    for push in jobs {
        let sent = match (push.platform, apple.as_mut(), google.as_mut()) {
            (Platform::Ios, Some(apple), _) => apple.send(&push),
            (Platform::Android, _, Some(google)) => google.send(&push),
            (platform, ..) => {
                info!("a push to {} ({platform:?}), with nothing set up to send it: {}", short(&push.token), push.body);
                continue;
            }
        };
        match sent {
            Sent::Delivered => debug!("pushed to {}", short(&push.token)),
            Sent::Gone => {
                info!("{} is nobody's any more: forgotten", short(&push.token));
                if found_gone.send(push.token).is_err() {
                    return;
                }
            }
            Sent::Failed => {}
        }
    }
}

/// Seconds since 1970.
fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |since| since.as_secs())
}

/// `json` as a JWT's header or claims are written: base64url, no padding.
fn part(json: &serde_json::Value) -> String {
    URL_SAFE_NO_PAD.encode(json.to_string())
}

/// What the environment says of `name`, if anything.
fn var(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

// ------------------------------------------------------------------- Apple

/// Apple's Push Notification service: the key, its id and the team's, the
/// app's bundle id, which side of the service, and the bearer signed last.
struct Apple {
    key: p256::ecdsa::SigningKey,
    key_id: String,
    team: String,
    topic: String,
    host: &'static str,
    bearer: Option<(String, Instant)>,
}

impl Apple {
    fn from_env() -> Option<Self> {
        let path = var("RT_APNS_KEY")?;
        let (Some(key_id), Some(team)) = (var("RT_APNS_KEY_ID"), var("RT_APNS_TEAM")) else {
            error!("RT_APNS_KEY without RT_APNS_KEY_ID and RT_APNS_TEAM: nothing is pushed to iPhones");
            return None;
        };
        let pem = match std::fs::read_to_string(&path) {
            Ok(pem) => pem,
            Err(error) => {
                error!("Apple's key, {path}, will not read ({error}): nothing is pushed to iPhones");
                return None;
            }
        };
        let key = match apple_key(&pem) {
            Some(key) => key,
            None => {
                error!("Apple's key, {path}, is no key of P-256: nothing is pushed to iPhones");
                return None;
            }
        };
        let sandbox = var("RT_APNS_SANDBOX").is_some_and(|on| on == "1");
        Some(Self {
            key,
            key_id,
            team,
            topic: var("RT_APNS_TOPIC").unwrap_or_else(|| TOPIC.into()),
            host: if sandbox { "api.sandbox.push.apple.com" } else { "api.push.apple.com" },
            bearer: None,
        })
    }

    /// The bearer to send with a push: the last one signed, while it is new
    /// enough, or another.
    fn bearer(&mut self) -> String {
        if let Some((bearer, signed)) = &self.bearer
            && signed.elapsed() < APPLE_BEARER_FOR
        {
            return bearer.clone();
        }
        let bearer = apple_bearer(&self.key, &self.key_id, &self.team, unix_now());
        self.bearer = Some((bearer.clone(), Instant::now()));
        bearer
    }

    fn send(&mut self, push: &Push) -> Sent {
        let payload = json!({ "aps": { "alert": { "body": push.body }, "sound": "default" } });
        let config = [
            format!("url = {}", quoted(&format!("https://{}/3/device/{}", self.host, push.token))),
            "http2".to_string(),
            format!("header = {}", quoted(&format!("authorization: bearer {}", self.bearer()))),
            format!("header = {}", quoted(&format!("apns-topic: {}", self.topic))),
            format!("header = {}", quoted("apns-push-type: alert")),
            format!("header = {}", quoted("apns-priority: 10")),
            format!("header = {}", quoted(&format!("apns-collapse-id: {}", push.kind))),
            format!("header = {}", quoted("content-type: application/json")),
            format!("data-binary = {}", quoted(&payload.to_string())),
        ];
        let Some((status, body)) = curl(&config) else {
            return Sent::Failed;
        };
        match status {
            200 => Sent::Delivered,
            // Unregistered, or never this app's.
            410 => Sent::Gone,
            400 if body.contains("BadDeviceToken") || body.contains("DeviceTokenNotForTopic") => Sent::Gone,
            _ => {
                // A bearer refused is signed afresh for the next.
                if status == 403 {
                    self.bearer = None;
                }
                warn!("Apple's service said {status} to a push: {}", body.trim());
                Sent::Failed
            }
        }
    }
}

/// Apple's key, from its `.p8` (PKCS #8, in PEM).
fn apple_key(pem: &str) -> Option<p256::ecdsa::SigningKey> {
    use p256::pkcs8::DecodePrivateKey;
    p256::ecdsa::SigningKey::from_pkcs8_pem(pem).ok()
}

/// A bearer for Apple's service, signed at `now` (seconds since 1970) with
/// `key`, whose id is `key_id`, of the team `team`: a JWT, ES256.
fn apple_bearer(key: &p256::ecdsa::SigningKey, key_id: &str, team: &str, now: u64) -> String {
    use p256::ecdsa::signature::Signer;
    let signed = format!("{}.{}", part(&json!({ "alg": "ES256", "kid": key_id })), part(&json!({ "iss": team, "iat": now })));
    let signature: p256::ecdsa::Signature = key.sign(signed.as_bytes());
    format!("{signed}.{}", URL_SAFE_NO_PAD.encode(signature.to_bytes()))
}

// ------------------------------------------------------------------ Google

/// Google's Firebase Cloud Messaging: the service account's key and
/// address, the project, and the access token asked for last, with when
/// it runs out.
struct Google {
    key: rsa::RsaPrivateKey,
    email: String,
    project: String,
    access: Option<(String, Instant)>,
}

/// The service account, as Firebase's console gives it: what of it is used.
#[derive(serde::Deserialize)]
struct Account {
    project_id: String,
    private_key: String,
    client_email: String,
}

impl Google {
    fn from_env() -> Option<Self> {
        let path = var("RT_FCM")?;
        let account: Account = match std::fs::read_to_string(&path).map(|text| serde_json::from_str(&text)) {
            Ok(Ok(account)) => account,
            Ok(Err(error)) => {
                error!("the service account, {path}, will not read ({error}): nothing is pushed to Android phones");
                return None;
            }
            Err(error) => {
                error!("the service account, {path}, will not open ({error}): nothing is pushed to Android phones");
                return None;
            }
        };
        let key = {
            use rsa::pkcs8::DecodePrivateKey;
            match rsa::RsaPrivateKey::from_pkcs8_pem(&account.private_key) {
                Ok(key) => key,
                Err(error) => {
                    error!("the service account's key will not read ({error}): nothing is pushed to Android phones");
                    return None;
                }
            }
        };
        Some(Self {
            key,
            email: account.client_email,
            project: account.project_id,
            access: None,
        })
    }

    /// An access token for the service: the last one asked for, while it
    /// has a while to go, or another, asked of Google's sign-in with a JWT
    /// the service account signs.
    fn access(&mut self) -> Option<String> {
        if let Some((token, until)) = &self.access
            && Instant::now() + GOOGLE_EARLY < *until
        {
            return Some(token.clone());
        }
        let assertion = google_assertion(&self.key, &self.email, unix_now());
        let config = [
            format!("url = {}", quoted("https://oauth2.googleapis.com/token")),
            format!(
                "data-binary = {}",
                quoted(&format!("grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={assertion}"))
            ),
        ];
        let (status, body) = curl(&config)?;
        let answer: serde_json::Value = serde_json::from_str(&body).ok()?;
        let (Some(token), Some(lasts)) = (answer["access_token"].as_str(), answer["expires_in"].as_u64()) else {
            warn!("Google's sign-in said {status}: {}", body.trim());
            return None;
        };
        self.access = Some((token.to_string(), Instant::now() + Duration::from_secs(lasts)));
        Some(token.to_string())
    }

    fn send(&mut self, push: &Push) -> Sent {
        let Some(access) = self.access() else {
            return Sent::Failed;
        };
        let message = json!({
            "message": {
                "token": push.token,
                "notification": { "body": push.body },
                "android": {
                    "priority": "high",
                    "notification": { "channel_id": ANDROID_CHANNEL, "tag": push.kind, "sound": "default" },
                },
            }
        });
        let config = [
            format!(
                "url = {}",
                quoted(&format!("https://fcm.googleapis.com/v1/projects/{}/messages:send", self.project))
            ),
            format!("header = {}", quoted(&format!("authorization: Bearer {access}"))),
            format!("header = {}", quoted("content-type: application/json")),
            format!("data-binary = {}", quoted(&message.to_string())),
        ];
        let Some((status, body)) = curl(&config) else {
            return Sent::Failed;
        };
        match status {
            200 => Sent::Delivered,
            // Unregistered: the app was deleted, or its token is old.
            404 => Sent::Gone,
            _ if body.contains("UNREGISTERED") => Sent::Gone,
            _ => {
                if status == 401 {
                    self.access = None;
                }
                warn!("Firebase said {status} to a push: {}", body.trim());
                Sent::Failed
            }
        }
    }
}

/// The JWT a service account signs to be given an access token for
/// Firebase's messaging, at `now` (seconds since 1970): RS256, an hour.
fn google_assertion(key: &rsa::RsaPrivateKey, email: &str, now: u64) -> String {
    use rsa::signature::{SignatureEncoding, Signer};
    let claims = json!({
        "iss": email,
        "scope": "https://www.googleapis.com/auth/firebase.messaging",
        "aud": "https://oauth2.googleapis.com/token",
        "iat": now,
        "exp": now + 3_600,
    });
    let signed = format!("{}.{}", part(&json!({ "alg": "RS256", "typ": "JWT" })), part(&claims));
    let signer = rsa::pkcs1v15::SigningKey::<sha2::Sha256>::new(key.clone());
    let signature = signer.sign(signed.as_bytes());
    format!("{signed}.{}", URL_SAFE_NO_PAD.encode(signature.to_bytes()))
}

// -------------------------------------------------------------------- curl

/// `value` as a curl config file has a value: in double quotes, with what
/// would end or bend it escaped.
fn quoted(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for c in value.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// Runs curl with `lines` as its config, on its standard input: what the
/// service answered, its status and its body. None if curl could not be run
/// or reached nothing.
fn curl(lines: &[String]) -> Option<(u16, String)> {
    let mut config = lines.join("\n");
    config.push('\n');
    config.push_str(&format!("write-out = {}\n", quoted("\n%{http_code}")));
    let mut child = match Command::new("curl")
        .args(["--silent", "--show-error", "--max-time", &CURL_SECS.to_string(), "--config", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            error!("curl will not run ({error}): nothing is pushed");
            return None;
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        // Dropped after, which ends what curl reads.
        let _ = stdin.write_all(config.as_bytes());
    }
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        warn!("curl: {}", String::from_utf8_lossy(&out.stderr).trim());
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let (body, status) = text.rsplit_once('\n')?;
    Some((status.trim().parse().ok()?, body.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bearer_for_apple_is_a_jwt_signed_with_its_key() {
        use p256::ecdsa::signature::Verifier;
        use p256::pkcs8::{EncodePrivateKey, LineEnding};
        // A key of P-256 made for this test alone, written as Apple's `.p8`
        // is, and read back as the server reads that.
        let made = p256::ecdsa::SigningKey::random(&mut rand_core_for_tests());
        let pem = made.to_pkcs8_pem(LineEnding::LF).expect("written");
        let key = apple_key(&pem).expect("the test key reads");
        assert_eq!(key, made);
        let bearer = apple_bearer(&key, "ABC123DEFG", "TEAM456789", 1_700_000_000);
        let pieces: Vec<&str> = bearer.split('.').collect();
        assert_eq!(pieces.len(), 3);
        let header: serde_json::Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(pieces[0]).unwrap()).unwrap();
        let claims: serde_json::Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(pieces[1]).unwrap()).unwrap();
        assert_eq!(header, json!({ "alg": "ES256", "kid": "ABC123DEFG" }));
        assert_eq!(claims, json!({ "iss": "TEAM456789", "iat": 1_700_000_000u64 }));
        // Its signature is r and s, 64 bytes, and holds for what it signs.
        let signature = URL_SAFE_NO_PAD.decode(pieces[2]).unwrap();
        assert_eq!(signature.len(), 64);
        let signature = p256::ecdsa::Signature::from_slice(&signature).unwrap();
        let signed = format!("{}.{}", pieces[0], pieces[1]);
        assert!(key.verifying_key().verify(signed.as_bytes(), &signature).is_ok());
        assert!(key.verifying_key().verify(b"something else", &signature).is_err());
    }

    #[test]
    fn what_google_s_sign_in_is_given_is_a_jwt_the_service_account_signed() {
        use rsa::pkcs1v15::{Signature, VerifyingKey};
        use rsa::signature::Verifier;
        let mut rng = rand_core_for_tests();
        let key = rsa::RsaPrivateKey::new(&mut rng, 1024).expect("a key");
        let assertion = google_assertion(&key, "pusher@project.iam.gserviceaccount.com", 1_700_000_000);
        let pieces: Vec<&str> = assertion.split('.').collect();
        assert_eq!(pieces.len(), 3);
        let claims: serde_json::Value = serde_json::from_slice(&URL_SAFE_NO_PAD.decode(pieces[1]).unwrap()).unwrap();
        assert_eq!(claims["iss"], "pusher@project.iam.gserviceaccount.com");
        assert_eq!(claims["scope"], "https://www.googleapis.com/auth/firebase.messaging");
        assert_eq!(claims["exp"].as_u64(), Some(1_700_003_600));
        let verifier = VerifyingKey::<sha2::Sha256>::new(key.to_public_key());
        let signature = Signature::try_from(URL_SAFE_NO_PAD.decode(pieces[2]).unwrap().as_slice()).unwrap();
        assert!(verifier.verify(format!("{}.{}", pieces[0], pieces[1]).as_bytes(), &signature).is_ok());
    }

    /// Dice for making a key in a test: the operating system's.
    fn rand_core_for_tests() -> rand_core::OsRng {
        rand_core::OsRng
    }

    #[test]
    fn what_curl_is_told_is_quoted_so_that_nothing_ends_it_early() {
        assert_eq!(quoted("plain"), "\"plain\"");
        assert_eq!(quoted(r#"{"a":"b\c"}"#), r#""{\"a\":\"b\\c\"}""#);
        assert_eq!(quoted("two\nlines"), "\"two\\nlines\"");
    }
}
