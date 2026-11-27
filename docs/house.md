# Houses: visits, ratings, the board, and pushes

> The account of houses being visited and rated, the House tab of the
> friends' panel, and the pushes to phones that go with them (since
> 2026-11-27). Dated entries, newest first. `CLAUDE.md` has the rules and
> how things stand; the quick chat's half of the same day is in
> `docs/chat-and-text.md`.

## "My House:" over your stats, and net worth as 28.2K (later on 2026-11-27)

Hajun, after APK 62: "in the house page, in my house stat page, i need text
above the stat boxes "My House:" and also for the numbe rin" ... "number in
the net worth thing, make it like 28.2K or 29.3M".

- **"My House:"** (`words.house.my_house`) stands over the six cards on
  the person's tab, in the panel's heading style, as "Ratings (n)" stands
  over the ratings; someone else's house from the board keeps its name and
  the chevron instead (mine: he asked of his own page).
- **Net worth** (`friends::house::worth`): as it is under a thousand, and
  past that in thousands, millions or billions to a tenth, in whichever
  keeps it under a thousand once rounded: 28.2K, 29.3M, 1.0K, and 1.0M
  where 999.95 thousand would round to 1000.0K. Always the tenth, as his
  examples have it (mine: "1.0K" rather than "1K"). The other cards are as
  they were.

Checked: the game's tests (`friends::house`, the numbers' cases); the
desktop build, hidden, against a server on this PC: the heading over the
cards (a new account's, all nought). **Not in an APK; nothing for the
server.**

## Visits, ratings, the House tab and pushes (2026-11-27)

In APK 62, and on the server deployed at 13:40:57 UTC that day (protocol
27; `docs/building.md`), at Hajun's "build apk and push". Pushes wait for
his keys (below).

Hajun, in one message with the quick chat's changes:

> add push notification for both android and ios - whenever someone visits
> your plot (stay for more than 10 seconds) it will send a push
> notification to you like "username is visiting your house. Go say hi!"
> after 15 seconds of staying on the plot, if you've never rated that
> persons plot's version before (if theyve modified the house after uve
> rated it, u can rate it again) u will get 5 stars at the bottom middle of
> ur screen, and when u tap it/select ur star, it will bring up a popup
> menu, with the star count that u tapped on the top (u can tap again to
> modify ur star betwen 0.5~5star with 0.5 increment) and also a list of
> rating adjectives that u can choose like "cute" "cool" "interesting"
> "pretty" "fancy" "good view" "amazing" "ugly" etc. and u can choose 0~ or
> up to 3 adjectives, and then at the bottom left u have cancel button in
> red and submit button in greeen on right. and that will give push
> notification to the owner like "Someone just rated your house. Open the
> app to see what rating they gave you!" and in the social tab, as the 3rd
> tab after the messages tab, it can be a house tab with a house icon and
> it has basic info about your house at the top like your house net worth,
> how many objects you have in your house, how many total visitors you got
> till now, how many visitors you got in the past 7 days, total amount of
> ratings you got, your total average rating, and then below it it can be a
> list of the ratings that ppl gave you where u can see their username and
> the star rating they gave u and the adjectives they chose.
>
> in the social popup, in the home tab, at the top i actually want another
> tab selector with 2 tabs (user icon on left, which is the thing i said
> before of stats and ratings of your own house, and leader board icon on
> the right). and in the leaderboard icon tab, u can see a global
> leaderboard of the best rated house that shows the top 100, and if you
> are in the top 100 ur name is on there, but if ur like top 101+ then it
> just shows u that ur place is top 101 on your side at the bottom, but not
> to anyone else, only the top 100 is shown to everyone. and for the top
> 100, u can scroll and click each one and u can see the stat & rating of
> that persons house, similar to how u see your own stat.
>
> keep the style of these new ui's as similar to the existing style as
> possible like the social panel and the inventory panel.

### What it does

- **Who is on whose plot is the server's to say** (`server/src/house.rs`,
  `track`, twice a second). Every plot is a 24 m square, turned as the map
  has it: their turns are written in `roundtown_net::house::TURNS` beside
  `land::HOMES`, and a test of the game's holds both to the map
  (`land::tests::the_square_the_server_takes_for_each_plot_is_the_map_s`).
  Whoever is online, in the town, on foot (not in a car nor on a bus) and
  inside the square of a plot whose land is someone else's
  (`Estate::owner`: whoever of the channel has it, here or away, or had it
  a moment ago, or whose real land fills it; never a made-up lot, never a
  bare plot) is staying there.
- **Ten seconds there is a visit** (`house::VISIT_SECS`): counted, and its
  owner told, "{name} is visiting your house. Go say hi!": across their
  screen if their app is open, pushed to their phone if not. **The same
  visitor counts again on the same house only an hour on**
  (`VISIT_AGAIN`), and **an owner's phone is pushed about visits at most
  once every two minutes** (`VISIT_PUSH_GAP`; every visit still counts).
- **Fifteen seconds there offers the stars** (`RATE_SECS`, `Tell::House
  (MayRate)`), unless the visitor has rated that land as it stands: the
  server keeps with every rating a print of what stands on the plot and
  its paint (`house::land_print`, FNV-1a over it all), and offers again
  once anything is put down, moved, turned, taken back or painted. Walking
  off takes the offer back; a rating sent up to two minutes after still
  counts (`OFFER_STANDS`).
- **The stars** (`src/rating.rs`) stand at the bottom middle of the
  screen on a plate of the buttons' grey, "Rate {name}'s house" over them.
  Each star is two halves to press: the left half is that star less a
  half. A press opens **the window**: the five stars along its top, big,
  pressed again by halves to change them, the number beside them; "Pick up
  to 3 words"; sixteen words in four rows of four, a press choosing one
  (gold) and another letting it go, a fourth doing nothing; Cancel (the
  window's dark red) bottom left and Submit (its green) bottom right. A
  press outside the window, or Escape, is Cancel. While it is up you stand
  still (`hud::DialogUp`). Submit sends `house::Ask::Rate`; the server
  keeps it, takes the stars away, says "Thanks for rating!", and tells the
  owner "Someone just rated your house!" across their screen, or pushes
  "Someone just rated your house. Open the app to see what rating they
  gave you!" to their phone.
- **One rating to a rater and a house**: a rating given again, for land
  changed since, takes the place of the one before.
- **The House tab** (`src/friends/house.rs`) is the third along the
  panel's top, Material's house. Under the panel's own tabs, two more,
  each half the width: the person and the leaderboard. **The person**: six
  cards in a row (net worth with the coin, objects, visitors, past 7 days,
  ratings, average with the star), and under them "Ratings (n)", the
  newest first: who (name and tag), their words under their name, five
  small stars and the number. **The leaderboard**: "Top 100 houses", a row
  to each, its place, whose, five small stars by the average rounded to a
  half, the average and how many ratings; your own row lit (the window's
  `row_mine`) with "(You)". A press on a row shows that house as yours is
  shown, its name over it and a chevron back. **Further down than a
  hundred**, your own row stands under the list, apart from it, for you
  alone (the server sends it only to you, `Board::mine`); with your house
  not rated at all, "Your house has not been rated yet." there instead.
  Everything in the tab is asked for as it is shown, and nothing is kept
  offline ("You are offline. Friends need a connection.").
- **The board's order** (`house::score`, `ranked`): the average weighted
  toward three stars as though every house had three more ratings of
  three, so that one five-star rating is not the best house in the game;
  then more ratings first; then the older account. Accounts deleted are
  left out of everything at once (by `social`'s own copy), and their rows
  go with them in the database (`ON DELETE CASCADE`).
- **What is kept**: three tables made by the server as it starts
  (`house::SCHEMA`, after the accounts'), written by a thread of the
  module's own with its own connection: `house_visits (owner, visitor,
  at)`, `house_ratings (rater, owner, halves, words, land, at)` (words as a
  bit to each, `words_to_bits`), `push_tokens (player, platform, token,
  at)`, a token being one account's at a time. All of it is read as the
  server starts; without a database it lasts as long as the server runs.
  None of it touches `accounts.rs`.

### Pushes

- **The server sends them** (`server/src/push.rs`), only to an owner whose
  app is not open (connected means open: a phone drops its connection as
  its app goes to the background); with it open, the same words go across
  their screen instead. **By curl**, the VM's own (Ubuntu's speaks HTTP/2,
  which Apple's service asks for), on a thread of its own; what curl is
  told, bearers among it, goes on its standard input, never its command
  line. The bearers are signed in plain Rust (`p256` for Apple's ES256,
  `rsa` for Google's RS256, `base64`), because the server is built for the
  VM with no C compiler (`deploy.sh`): it still cross-compiles, checked.
  A later push of the same kind replaces the one before on the phone
  (APNs' collapse id, FCM's tag: "visit", "rated"). A token a service says
  is nobody's (APNs 410 or BadDeviceToken, FCM 404 or UNREGISTERED) is
  forgotten. **Without the keys nothing is pushed**, and the server logs
  each push it would have sent ("a push to … with nothing set up to send
  it").
- **Android** (`MainActivity`, `src/push.rs`): Firebase Cloud Messaging
  24.0.0. As the app opens it makes the channel "Your house" and asks
  Firebase for the phone's token, which it hands to the game a byte at a
  time (`pushTokenBegin`, `pushTokenByte`, `pushTokenEnd`); the game tells
  the server as it comes online. The first time you stand in the town
  online after the app opens, the game has the activity ask Android 13 and
  later whether word may be shown (`pushWanted`, `POST_NOTIFICATIONS`).
  The small icon is Material's house, white (`res/drawable/
  ic_notification.xml`), tinted the stars' gold. **The Gradle plugin is
  applied only if `mobile/android/app/google-services.json` is there**, so
  the app builds and plays without it, just never given a token.
- **iPhone** (`src/push.rs`, `mod ios`; `mobile/ios/RoundTown.entitlements`,
  `aps-environment` production, named in `project.yml`): winit gives UIKit
  no app delegate, so the game sets one of its own (`PushDelegate`, an
  Objective-C class declared with `objc2` 0.5, the version winit already
  builds there) that hears the token and nothing else, at the same moment
  Android's question is asked; and asks `UNUserNotificationCenter` for
  alerts, sounds and badges. **Type-checked only, never run**: if it goes
  wrong, the phone's `log.txt` says so first ("Apple has given this phone
  its token", "no push token from Apple: …").
- On desktop `RT_PUSH_TOKEN=android:<token>` or `ios:<token>` makes up a
  token, for trying the server's side.

### Setting the pushes up (Hajun's, once)

None of this could be done from here: each needs Hajun's own accounts.

1. **Firebase** (Android): at console.firebase.google.com, a project
   (any name), and in it an Android app with the package name
   `town.donggeurami.app`. Download its `google-services.json` into
   `mobile/android/app/` (it is in `.gitignore`: the repository is
   public). Then, in the project's settings, Service accounts, "Generate
   new private key": a JSON file that is the server's key. **Never put it
   in the repository.**
2. **Apple** (iPhone), at developer.apple.com, Certificates, Identifiers
   & Profiles: under Identifiers, `town.donggeurami.app`, tick **Push
   Notifications** and save; under Profiles, delete the App Store profile
   of `town.donggeurami.app`, so that Codemagic makes a new one with push in
   it at the next build (**until both are done, the next `ios-*` tag fails
   at signing**, the app now asking for `aps-environment`); under Keys, a
   new key with **Apple Push Notifications service (APNs)**, its `.p8`
   downloaded (once only), its Key ID, and the Team ID (Membership).
3. **The VM**: the two files in `/etc/roundtown/` (`fcm.json`,
   `AuthKey_XXXXXXXXXX.p8`), owned by root and readable by the group
   `roundtown` (`chmod 640`), and in `/etc/roundtown/env`:
   `RT_FCM=/etc/roundtown/fcm.json`, `RT_APNS_KEY=/etc/roundtown/
   AuthKey_XXXXXXXXXX.p8`, `RT_APNS_KEY_ID=XXXXXXXXXX`,
   `RT_APNS_TEAM=YYYYYYYYYY`; then a restart (`systemctl restart
   roundtown-server`). The log says "pushes go to iPhones and Android
   phones" as it starts. `curl -V` on the VM has to list HTTP2.
4. A new APK, built with the `google-services.json` in place; and the
   `ios-*` tag, after step 2.

### Calls that were mine

- **One rating to a rater and a house**, the latest; ratings ranked by an
  average weighted toward three stars over three ratings; ties by more
  ratings, then the older account.
- **A visit counts once an hour** for the same visitor and house; pushes
  about visits at most every two minutes to one owner; nothing pushed to
  an owner whose app is open (told across the screen instead), and the
  same words across the screen ("Someone just rated your house!" there,
  the push's second sentence being about opening the app).
- "Thanks for rating!" across the rater's screen; "Rate {name}'s house"
  over the stars; the number beside the window's stars; a press outside
  the window is Cancel; a fourth word does nothing.
- **The words**: Hajun's eight first, in his order (Cute, Cool,
  Interesting, Pretty, Fancy, Good view, Amazing, Ugly), then eight of
  mine: Cozy, Creative, Unique, Spacious, Colorful, Relaxing, Messy,
  Boring. Four rows of four.
- Visits are counted as visits, not as people: a friend coming back the
  next day counts again.
- The plot is its own 24 m square, not the 17 m circle the server used
  for taking plots; in a car or on a bus nothing counts.
- When the phone is asked whether word may be shown: the first time you
  stand in the town online after the app opens. The channel's name, "Your
  house". The icon, Material's house, and the gold.
- Firebase Messaging 24.0.0, the last for compileSdk 34; no analytics.
- Push by the VM's curl rather than TLS in the server: no C compiler.

### Checked

Unit tests: `roundtown_net` (the squares against each other and the old
circle, words, the board's order, the land's print, tokens; `Estate::owner`),
the server's (visits once an hour and the week, a rating replaced and the
deleted left out, a token one account's, what is kept read back; the
bearers for Apple and Google signed and verified with keys made in the
test), the game's (the stars by halves, three words, the file's sixteen
words; the House tab's numbers and its back and forth; the squares against
the map). The server cross-compiles for the VM. Android's Java compiles
against Firebase (`gradlew :app:compileReleaseJavaWithJavac`). The
iPhone's build type-checks (`mobile/ios/check_on_windows.sh`). Clippy has
nothing in the new code. Whole suites: `roundtown_net` 310, the server 142,
the game 630.

**Three copies against a server on this PC** (no database, hidden, a
throwaway driver taken out again, `RT_PUSH_TOKEN` made up for the owner):
Mochi said "Hi!" (the fan in two columns; the bubble over her head and
"Mochi: Hi!" in both copies' chat); Dubu walked onto Mochi's plot; ten
seconds on, the server counted the visit and Mochi, online, had "Dubu is
visiting your house. Go say hi!" across her screen and no push; at 15.3 s
Dubu had the stars ("Rate Mochi's house"); a press on the fourth star's
right half opened the window at 4.0, Cute and Cozy were chosen, the fourth
star's left half made it 3.5, Submit: the server kept 7 halves, Dubu's
stars went with "Thanks for rating!", and Mochi had "Someone just rated
your house!"; Dubu said "Nice house!", over his head for Mochi too. Mochi's
House tab: 0 coins, 0 objects, 1 visitor, 1 in the week, 1 rating, 3.5,
and Dubu#… with "Cute · Cozy" and three and a half stars; the board, "#1
Mochi (You) 3.5 (1)" lit; a press on it, "Mochi's house" with a chevron
back. Mochi then quit, and a third copy, Bori, stood on her plot (kept for
her a while, `land::GRACE`): the visit counted and the server pushed, as
far as it can without keys ("a push to test-tok (Android), with nothing set
up to send it: Bori is visiting your house. Go say hi!"), and Bori had the
stars for Mochi's land. Seen in the pictures and put right: the words three
to a row (now four), and the bubble's words smaller than the names over
heads (now half as big again as a name, and never bigger on the screen
than from 8 m, as a name).

**Not checked**: anything on a phone; a push actually delivered (no
Firebase project, no APNs key yet); the iPhone's code, which is only
type-checked; real accounts and the database on the VM (the tables have
not been made against Postgres yet); more than a hundred houses on the
board.
