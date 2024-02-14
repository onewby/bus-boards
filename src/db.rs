use std::sync::Arc;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Params, params};

pub type DBPool = Pool<SqliteConnectionManager>;
pub type PooledConn = PooledConnection<SqliteConnectionManager>;

pub fn open_db() -> Pool<SqliteConnectionManager> {
    let manager = SqliteConnectionManager::file("/Users/onewby/Documents/BetterBuses/bus-site/stops.sqlite");
    Pool::new(manager).unwrap()
}

pub fn get_pool(db: &Arc<DBPool>) -> PooledConn {
    let mut conn: Result<PooledConnection<SqliteConnectionManager>, r2d2::Error>;
    while {
        conn = db.get();
        conn.is_err()
    } {}
    eprintln!("Database access timeout!");
    conn.unwrap()
}

pub fn get_string<P: Params>(db: &Arc<DBPool>, query: &str, params: P) -> rusqlite::Result<String> {
    get_pool(db).prepare(query).unwrap().query_row(params, |aid| aid.get(0))
}

pub fn get_agency<'a>(db: &Arc<DBPool>, code: &str) -> rusqlite::Result<String> {
    get_string(db, "SELECT agency_id FROM traveline WHERE code=?", params![code])
}

pub fn get_route<'a>(db: &Arc<DBPool>, code: &str, route: &str) -> rusqlite::Result<String> {
    get_string(db, "SELECT route_id FROM routes INNER JOIN main.traveline t on routes.agency_id = t.agency_id WHERE code=? AND route_short_name=?", (code, route))
}