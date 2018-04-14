use rusqlite::Connection;
use ntfs::FileEntry;
use rusqlite::Transaction;

const INSERT_FILE: &str = "INSERT INTO file_entry (id, fr_number, parent_fr, flags, real_size, logical_size, name, modified_date, created_date) \
    VALUES (:id, :fr_number, :parent_fr, :flags, :real_size, :logical_size, :name, :modified_date, :created_date)";
const UPDATE_FILE: &str = "UPDATE file_entry SET \
    id = :id, fr_number = :fr_number, parent_fr = :parent_fr, flags = :flags, real_size = :real_size, logical_size = :logical_size, name = :name, modified_date = :modified_date, created_date = :created_date \
    WHERE id = :id;";
const DELETE_FILE: &str = "DELETE FROM file_entry WHERE id = :id;";

pub fn main() -> Connection {
    let conn = Connection::open("test.db").unwrap();

    conn.query_row("PRAGMA encoding;", &[], |row| {
        let x: String = row.get(0);
        println!("{}", x);
    }).unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS file_entry (
                  _id           INTEGER PRIMARY KEY,
                  id            INTEGER UNIQUE,
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
    conn.prepare(UPDATE_FILE).unwrap();
    conn.prepare(DELETE_FILE).unwrap();
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
}

pub fn delete_file(tx: &Transaction, file_id: u16) {
    tx.execute_named(DELETE_FILE, &[
        (":id", &file_id)]).unwrap();
}


pub fn update_file(tx: &Transaction, file: &FileEntry) {
    tx.execute_named(UPDATE_FILE, &[
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

pub fn insert_files(connection: &mut Connection, files: &[FileEntry]) {
    let tx = connection.transaction().unwrap();
    for file in files {
        insert_file(&tx, file);
    }
    tx.commit().unwrap();
}