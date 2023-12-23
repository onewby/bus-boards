import Database from 'better-sqlite3';

export const db = new Database("stops.sqlite"/*, {readonly: true}*/);
db.pragma('journal_mode = WAL');