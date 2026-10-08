# The server — running 동그라미타운 online

How the game server is run: where it is, how a change gets onto it, what to
look at when something is wrong. `MULTIPLAYER.md` is the plan and the why;
this is the how, as built on 2026-10-08.

---

## What runs where

One Vultr VM in Seoul (`roundtown-seoul-1`, plan `vc2-1c-2gb`, $10 a month):
Ubuntu 24.04, 1 vCPU, 2 GB, IPv4 `141.164.62.193`, IPv6
`2401:c080:1c02:dd6:5400:6ff:fed3:eff6`. On it:

| What | Where | Port |
|---|---|---|
| The game server, `roundtown-server` (this repo's `server/`) | systemd `roundtown-server.service`, as user `roundtown` | UDP 443 (the game), TCP 127.0.0.1:8080 (the login) |
| Caddy, for HTTPS | systemd `caddy.service`, `/etc/caddy/Caddyfile` | TCP 443, in front of the login |
| Postgres 16 | systemd `postgresql.service`, database `roundtown` | the local socket only |
| The nightly backup | systemd `roundtown-backup.timer`, 19:00 UTC (04:00 in Korea) | — |

Port 443 for the game because a network that lets any UDP through lets that
through (QUIC uses it): school and office networks often stop the rest. Caddy
has HTTP/3 turned off so that UDP 443 stays the game's.

The VM's own Vultr Auto Backups (if they were turned on when it was made)
carry everything, the nightly database dumps included, off the machine.

## Getting in

1. A device makes up a secret the first time it runs: sixteen random bytes,
   `device.id`, kept beside `town.json`.
2. It posts the secret to `https://<address>/login` (`src/login.rs`). The
   certificate there is the server's own, self-signed, for ten years. The
   app trusts that certificate and nothing else, by its SHA-256
   (`login::PINNED`), so no domain is needed.
3. The login (`server/src/login.rs`) finds the account the secret's hash
   belongs to, or makes one (a generated name, 1,000 coins), and answers with
   a netcode connect token for the game, naming the account and signed with
   `RT_KEY`.
4. The device connects to UDP 443 with the token. Without one from the login,
   nobody gets in: the server is in netcode's secure mode.

The app tries the IPv6 address first and the IPv4 one after (`net::SERVERS`):
Apple tests apps on IPv6-only networks. A network without IPv6 fails at once,
so it costs nothing there.

**The addresses and the certificate are compiled into the app.** A new VM
means a new certificate (and new addresses): update `net::SERVERS` and
`login::PINNED` and ship a new app. A domain later (it is wanted anyway
for the privacy policy and the account-deletion page) can carry a
Let's Encrypt certificate instead; the app would then trust the usual
authorities for that name.

## Deploying

From Git Bash on the Windows PC, in the repo:

```
server/deploy.sh            # build for Linux, upload, restart (about 2 min)
server/deploy.sh --logs     # the server's log, following it
```

It cross-compiles: the server is pure Rust, so rustup's musl target and
`rust-lld` make a static Linux binary on Windows with no C toolchain or
Docker (`rustup target add x86_64-unknown-linux-musl`, done once). It logs
in with the key made for the VM, `~/.ssh/roundtown_vultr`.

**Deploy the server with every change to `shared/`.** The app and the server
register the same things in the same order; replicon hashes that, and an app
whose hash differs from the server's is told it is out of date ("A new
version is out") and stays offline. The hash is only the names of the types
registered and their order, not what is in them: bump `PROTOCOL_VERSION` in
`shared/` whenever a field is added to anything sent, taken out, or made to
mean something else, or an old app connects and breaks instead. Phones keep
the app they have until they update, so think twice before changing what is
sent once real players have it. The prizes (deployed 2026-10-09) changed
nothing sent, so the apps already out kept playing online with the new
server.

A fresh VM is set up with `server/setup.sh`: packages, the `roundtown` user,
the firewall, SSH by key only, the certificate, the settings, the database,
Caddy and the backup timer. It is safe to run again; it leaves what is there
alone.

## Settings

`/etc/roundtown/env` (readable by root and the server's group):

| Setting | What |
|---|---|
| `RT_PORT=443` | The game's UDP port |
| `RT_KEY` | 64 hex digits, the key tokens are signed with. Made by `setup.sh`; it never leaves the VM, and the app never has it. Changing it costs nothing but the tokens handed out in the last two minutes |
| `RT_PUBLIC` | The addresses a token may name: the VM's IPv4 and IPv6, port 443 |
| `RT_DB` | `host=/var/run/postgresql user=roundtown dbname=roundtown`: Postgres over its local socket, no password (peer authentication) |
| `RT_ALLOW_SKIP=1` | Lets a device skip House Builder's build and visits (the tap on the time). **On for testing: remove it before real players come** |
| `NO_COLOR=1` | Plain text in the log |

After changing it: `systemctl restart roundtown-server`.

## Looking at it

```
ssh -i ~/.ssh/roundtown_vultr root@141.164.62.193
journalctl -u roundtown-server -f          # who joins, leaves, plays; once a minute, traffic
systemctl status roundtown-server caddy postgresql
sudo -u roundtown psql -d roundtown        # the database
```

Once a minute, while anyone is on, the server logs how many are connected and
where, and the bytes going out and in, all told and per player: compare with
`MULTIPLAYER.md`'s tables. Measured on 2026-10-08 with two players: 0.4–0.8
KB/s out per player, most of it renet keeping the connection alive.

## The database

```
players       id, name, look, coins, created, seen
devices       secret (SHA-256 of the device's secret), player
coin_changes  player, amount, reason, game, at
```

The server makes the tables when it starts (`accounts::SCHEMA`). Coins change
only in one transaction with a line in `coin_changes` saying why: so far
`starting balance` (+1,000, as the login makes the account), `House
Builder entry` (-100) and `House Builder prize` (+500 down to +25, by
place; 8th place pays nothing and writes no line). A prize carries its
game's number; a fee, the number of the game filling when it was paid, and
none if none was yet. Deleting an account (in the app: the button beside
the balance) deletes its row, and its devices and coin history with it.

**Backups:** `/var/backups/roundtown/`, the last 14 nights, root only. To
restore one:

```
systemctl stop roundtown-server
sudo -u postgres dropdb roundtown && sudo -u postgres createdb -O roundtown roundtown
sudo -u roundtown pg_restore --dbname=roundtown < /var/backups/roundtown/<file>.dump
systemctl start roundtown-server
```

A restore was tried on 2026-10-08, into a scratch database, and matched.

## When something is wrong

- **Nobody can connect.** `systemctl status roundtown-server caddy`. `ufw
  status` should allow 443/udp and 443/tcp. From a PC, `curl -k -X POST
  https://141.164.62.193/login` should answer 400 (a request with nothing in
  it); 502 means Caddy is up and the server is not.
- **One phone cannot connect, others can.** Its network may block UDP. The
  app keeps trying, quietly, and plays offline meanwhile.
- **"A new version is out".** The app and the server were built from
  different `shared/`: deploy the server, or update the app.
- **The server will not start.** `journalctl -u roundtown-server -n 50`. It
  stops on purpose without the database, rather than take anyone's coins.
