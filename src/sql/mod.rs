use rusqlite::Connection;
use ntfs::FileEntry;
use std::time::Instant;
use rusqlite::Transaction;

const INSERT_FILE: &str = "INSERT INTO file_entry (id, fr_number, parent_fr, flags, real_size, logical_size, name, modified_date, created_date) \
    VALUES (:id, :fr_number, :parent_fr, :flags, :real_size, :logical_size, :name, :modified_date, :created_date)";
pub fn main() -> Connection {
    let mut conn = Connection::open("test.db").unwrap();

    conn.query_row("PRAGMA encoding;", &[], |row| {
        let x: String = row.get(0);
        println!("{}", x);
    }).unwrap();

    conn.execute("CREATE TABLE file_entry (
                  _id           INTEGER PRIMARY KEY,
                  id            INTEGER,
                  fr_number     INTEGER,
                  parent_fr     INTEGER,
                  flags         INTEGER,
                  real_size     INTEGER,
                  logical_size  INTEGER,
                  name          TEXT,
                  modified_date INTEGER,
                  created_date  INTEGER
                  )", &[]).unwrap();
    conn.prepare(INSERT_FILE).unwrap();
    conn

//    let now = Instant::now();
//    insert_file(&mut conn, &FileEntry::default(), 500000);
//    println!("total time {:?}", Instant::now().duration_since(now));
//
//    let mut stmt = conn.prepare("SELECT count(*) FROM file_entry").unwrap();
//    let count: i64 = stmt.query_row(&[], |r| r.get(0)).unwrap();
//    println!("Added {} files", count);
}

pub fn insert_file(tx: &Transaction, file: &FileEntry) {
    tx.execute_named(INSERT_FILE, &[
        (":id", &file.id),
        (":fr_number", &file.fr_number),
        (":parent_fr", &file.parent_fr),
        (":flags", &file.dos_flags),
        (":real_size", &file.real_size),
        (":logical_size", &file.logical_size),
        (":name", &file.name),
        (":modified_date", &file.modified_date),
        (":created_date", &file.created_date),
    ]).unwrap();

//    transaction.commit().unwrap();
}