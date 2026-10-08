#!/bin/sh
# Nightly copy of the database (MULTIPLAYER.md, phase 2), run as root by
# roundtown-backup.timer. Keeps the last 14 in /var/backups/roundtown, readable
# by root alone; the VM's own Vultr backups carry them off the machine.
#
# To restore one:
#   systemctl stop roundtown-server
#   sudo -u postgres dropdb roundtown && sudo -u postgres createdb -O roundtown roundtown
#   sudo -u roundtown pg_restore --dbname=roundtown < /var/backups/roundtown/<file>.dump
#   systemctl start roundtown-server
set -eu
umask 077
dir=/var/backups/roundtown
mkdir -p "$dir"
stamp=$(date -u +%Y-%m-%dT%H%MZ)
sudo -u roundtown pg_dump --format=custom --dbname=roundtown > "$dir/roundtown-$stamp.dump.part"
mv "$dir/roundtown-$stamp.dump.part" "$dir/roundtown-$stamp.dump"
ls -1t "$dir"/roundtown-*.dump | tail -n +15 | xargs -r rm --
