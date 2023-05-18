#!/usr/bin/env sh

rm -f bus-site/stops.sqlite bus-site/stops.sqlite-shm bus-site/stops.sqlite-wal
python3 localities.py
cd bus-site
npx ts-node-esm import_gtfs.ts