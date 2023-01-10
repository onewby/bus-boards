import Database from 'better-sqlite3';

export const db = new Database("stops.sqlite");
db.pragma('journal_mode = WAL');