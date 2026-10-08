#!/usr/bin/env bash
# Sets up a fresh Ubuntu 24.04 VM to run the game server: what was done by
# hand on the Vultr VM in Seoul on 2026-10-08, written down so that it can be
# done again. Safe to run more than once: each step leaves what is already
# there alone. Then `server/deploy.sh` puts the server itself on it.
#
#   server/setup.sh                      the VM at 141.164.62.193
#   RT_HOST=root@<ip> server/setup.sh    another one
#
# What it does, as root on the VM:
#
# * Packages: Postgres (16) and Caddy, and the system brought up to date.
# * A user for the server, `roundtown`, with no shell, and /opt/roundtown for
#   its program, /etc/roundtown for its settings, /var/lib/roundtown for it to
#   work in.
# * The firewall (ufw, on in Vultr's images): SSH, and 443 over UDP for the
#   game and over TCP for the login. SSH by key only from then on.
# * The login's certificate, self-signed for the VM's addresses, for ten
#   years. A new VM means a new certificate, and its SHA-256, printed at the
#   end, goes into the app as `login::PINNED`, with the addresses into
#   `net::SERVERS`: a new app.
# * /etc/roundtown/env: the key tokens are signed with (made here, once),
#   the addresses, and the database.
# * The database, `roundtown`, which the server's user reaches over the local
#   socket with no password.
# * Caddy, for HTTPS (server/ops/Caddyfile), and the nightly backup
#   (server/ops/roundtown-backup.*).
#
# The VM's addresses are read off the VM itself.

set -euo pipefail

HOST="${RT_HOST:-root@141.164.62.193}"
KEY="${RT_KEY:-$HOME/.ssh/roundtown_vultr}"

cd "$(dirname "$0")"

scp -i "$KEY" -o BatchMode=yes ops/Caddyfile ops/roundtown-backup.sh \
    ops/roundtown-backup.service ops/roundtown-backup.timer "$HOST:/tmp/"

ssh -i "$KEY" -o BatchMode=yes "$HOST" 'bash -s' <<'EOF'
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive

v4=$(ip -4 -o addr show scope global | awk '{print $4}' | cut -d/ -f1 | head -1)
v6=$(ip -6 -o addr show scope global | awk '{print $4}' | cut -d/ -f1 | head -1)
echo "addresses: $v4 $v6"

apt-get update -q >/dev/null
apt-get -y -q -o Dpkg::Options::=--force-confold upgrade >/dev/null
apt-get -y -q install postgresql caddy >/dev/null

id roundtown >/dev/null 2>&1 || useradd --system --home-dir /var/lib/roundtown \
    --create-home --shell /usr/sbin/nologin roundtown
mkdir -p /opt/roundtown /etc/roundtown
chmod 755 /etc/roundtown

ufw allow 22/tcp >/dev/null
ufw allow 443/udp comment 'roundtown game' >/dev/null
ufw allow 443/tcp comment 'roundtown login' >/dev/null
ufw --force enable >/dev/null

cat > /etc/ssh/sshd_config.d/10-roundtown.conf <<'CONF'
PasswordAuthentication no
KbdInteractiveAuthentication no
PermitRootLogin prohibit-password
CONF
sshd -t && systemctl reload ssh

cd /etc/roundtown
if [ ! -f login.key ]; then
    san="IP:$v4"
    [ -n "$v6" ] && san="$san,IP:$v6"
    openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes \
        -keyout login.key -out login.crt -days 3650 -subj "/CN=$v4" \
        -addext "subjectAltName=$san" 2>/dev/null
fi
chown root:caddy login.key login.crt
chmod 640 login.key

touch env
grep -q '^RT_PORT=' env || echo 'RT_PORT=443' >> env
grep -q '^NO_COLOR=' env || echo 'NO_COLOR=1' >> env
grep -q '^RT_KEY=' env || echo "RT_KEY=$(openssl rand -hex 32)" >> env
public="$v4:443"
[ -n "$v6" ] && public="$public,[$v6]:443"
grep -q '^RT_PUBLIC=' env || echo "RT_PUBLIC=$public" >> env
grep -q '^RT_DB=' env || echo 'RT_DB=host=/var/run/postgresql user=roundtown dbname=roundtown' >> env
chown root:roundtown env
chmod 640 env

sudo -u postgres psql -qtc "SELECT 1 FROM pg_roles WHERE rolname='roundtown'" | grep -q 1 \
    || sudo -u postgres createuser roundtown
sudo -u postgres psql -qtc "SELECT 1 FROM pg_database WHERE datname='roundtown'" | grep -q 1 \
    || sudo -u postgres createdb -O roundtown roundtown

sed "s/141.164.62.193/$v4/" /tmp/Caddyfile > /etc/caddy/Caddyfile
caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile >/dev/null 2>&1
systemctl restart caddy

install -m 755 /tmp/roundtown-backup.sh /opt/roundtown/roundtown-backup.sh
install -m 644 /tmp/roundtown-backup.service /tmp/roundtown-backup.timer /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now roundtown-backup.timer >/dev/null
rm -f /tmp/Caddyfile /tmp/roundtown-backup.*

echo "the login's certificate, SHA-256 (login::PINNED):"
openssl x509 -in login.crt -outform DER | sha256sum | cut -d' ' -f1
echo "set up: now run server/deploy.sh"
EOF
