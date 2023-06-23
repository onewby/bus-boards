#!/usr/bin/env sh

rm -f bus-site/stops.sqlite bus-site/stops.sqlite-shm bus-site/stops.sqlite-wal
python3 localities.py
cd bus-site
tsc --esModuleInterop --module esnext --target esnext --moduleResolution node import_gtfs.ts
node import_gtfs.js