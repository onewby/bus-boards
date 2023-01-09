import Database from 'better-sqlite3';
import { fileURLToPath } from 'node:url';
import {parse as parsePath} from "node:path";

const __file = parsePath(fileURLToPath(import.meta.url))

export const db = new Database(__file.dir + "/../stops.sqlite");
db.pragma('journal_mode = WAL');