use ntfs::FileEntry;
use rusqlite::Connection;
use rusqlite::Transaction;
use rusqlite::Row;
use rusqlite::Result;

const INSERT_FILE: &str = "INSERT INTO file_entry (id, parent_id, flags, real_size, logical_size, name, modified_date, created_date) \
    VALUES (:id, :parent_id, :flags, :real_size, :logical_size, :name, :modified_date, :created_date);";
const UPSERT_FILE: &str = "INSERT OR REPLACE INTO file_entry (id, parent_id, flags, real_size, logical_size, name, modified_date, created_date) \
    VALUES (:id, :parent_id, :flags, :real_size, :logical_size, :name, :modified_date, :created_date);";
const UPDATE_FILE: &str = "UPDATE file_entry SET \
    id = :id, parent_id = :parent_id, flags = :flags, real_size = :real_size, logical_size = :logical_size, name = :name, modified_date = :modified_date, created_date = :created_date \
    WHERE id = :id;";
const DELETE_FILE: &str = "DELETE FROM file_entry WHERE id = :id;";
const COUNT_FILES: &str = "SELECT COUNT(id) FROM file_entry where name like :name";

pub fn main() -> Connection {
    let conn = Connection::open("test.db").unwrap();

    conn.query_row("PRAGMA encoding;", &[], |row| {
        let x: String = row.get(0);
        println!("{}", x);
    }).unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS file_entry (
                  id            INTEGER PRIMARY KEY,
                  parent_id     INTEGER,
                  flags         INTEGER,
                  real_size     INTEGER,
                  logical_size  INTEGER,
                  name          TEXT,
                  modified_date INTEGER,
                  created_date  INTEGER
                  );", &[]).unwrap();
    conn.prepare_cached(INSERT_FILE).unwrap();
    conn.prepare_cached(UPDATE_FILE).unwrap();
    conn.prepare_cached(DELETE_FILE).unwrap();
    conn.prepare_cached(UPSERT_FILE).unwrap();
    conn.prepare_cached(COUNT_FILES).unwrap();
    conn

//    let now = Instant::now();
//    insert_file(&mut conn, &FileEntry::default(), 500000);
//    println!("total time {:?}", Instant::now().duration_since(now));
//
//    let mut stmt = conn.prepare("SELECT count(*) FROM file_entry").unwrap();
//    let count: i64 = stmt.query_row(&[], |r| r.get(0)).unwrap();
//    println!("Added {} files", count);
}

//fn insert_file(tx: &Transaction, file: &FileEntry) {
//    tx.execute_named(INSERT_FILE, &[
//        (":id", &file.id),
//        (":parent_id", &file.parent_id),
//        (":flags", &file.dos_flags),
//        (":real_size", &file.real_size),
//        (":name", &file.name),
//        (":modified_date", &file.modified_date),
//        (":created_date", &file.created_date),
//    ]).unwrap();
//}

pub fn delete_file(tx: &Transaction, file_id: u32) {
    tx.execute_named(DELETE_FILE, &[
        (":id", &file_id)]).unwrap();
}

pub fn upsert_file(tx: &Transaction, file: &FileEntry) {
    tx.execute_named(UPSERT_FILE, &[
        (":id", &file.id),
        (":parent_id", &file.parent_id),
        (":flags", &file.dos_flags),
        (":real_size", &file.real_size),
        (":logical_size", &file.logical_size),
        (":name", &file.name),
        (":modified_date", &file.modified_date),
        (":created_date", &file.created_date),
    ]).unwrap();
}

pub fn update_file(tx: &Transaction, file: &FileEntry) {
    tx.execute_named(UPDATE_FILE, &[
        (":id", &file.id),
        (":parent_id", &file.parent_id),
        (":flags", &file.dos_flags),
        (":real_size", &file.real_size),
        (":logical_size", &file.logical_size),
        (":name", &file.name),
        (":modified_date", &file.modified_date),
        (":created_date", &file.created_date),
    ]).unwrap();
}

pub fn insert_files(connection: &mut Connection, files: &[FileEntry]) {
    let tx = connection.transaction().unwrap();
    {
        let mut stmt = tx.prepare_cached(INSERT_FILE).unwrap();
        for file in files {
            stmt.execute_named(
                &[
                    (":id", &file.id),
                    (":parent_id", &file.parent_id),
                    (":flags", &file.dos_flags),
                    (":real_size", &file.real_size),
                    (":logical_size", &file.logical_size),
                    (":name", &file.name),
                    (":modified_date", &file.modified_date),
                    (":created_date", &file.created_date)]
            ).unwrap();
        }
    }
    tx.commit().unwrap();
}

pub fn count_files(con: &Connection, name: &String) -> u32 {
    let mut statement = con.prepare_cached(COUNT_FILES).unwrap();
    let handle_row = |row: &Row| -> Result<u32> { Ok(row.get(0))};
    let mut result= statement.query_and_then_named(&[(":name", name)], handle_row).unwrap();
    result.nth(0).unwrap().unwrap()
}
