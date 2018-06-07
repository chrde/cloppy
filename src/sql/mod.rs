use file_listing::file_entity::FileEntity;
use ntfs::FileEntry;
use rusqlite::Connection;
use rusqlite::Result;
use rusqlite::Row;
use rusqlite::Transaction;
use sql::arena::Arena;

pub mod arena;

const CREATE_DB: &str = "
    CREATE TABLE IF NOT EXISTS file_entry (
    _id           INTEGER PRIMARY KEY,
    id            INTEGER,
    parent_id     INTEGER,
    dos_flags     INTEGER,
    real_size     INTEGER,
    name          TEXT,
    modified_date INTEGER,
    created_date  INTEGER,
    flags         INTEGER,
    base_record   INTEGER,
    fr_number     INTEGER,
    namespace     INTEGER );
    ";
const INSERT_FILE: &str = "INSERT INTO file_entry (id, parent_id, dos_flags, real_size, name, modified_date, created_date, flags, base_record, fr_number, namespace) \
    VALUES (:id, :parent_id, :dos_flags, :real_size, :name, :modified_date, :created_date, :flags, :base_record, :fr_number, :namespace);";
const UPSERT_FILE: &str = "INSERT OR REPLACE INTO file_entry (id, parent_id, dos_flags, real_size, name, modified_date, created_date) \
    VALUES (:id, :parent_id, :dos_flags, :real_size, :name, :modified_date, :created_date);";
const UPDATE_FILE: &str = "UPDATE file_entry SET \
    id = :id, parent_id = :parent_id, dos_flags = :dos_flags, real_size = :real_size, name = :name, modified_date = :modified_date, created_date = :created_date \
    WHERE id = :id;";
const DELETE_FILE: &str = "DELETE FROM file_entry WHERE id = :id;";
const COUNT_FILES: &str = "SELECT COUNT(id) FROM file_entry where name like :name";
const SELECT_FILES: &str = "SELECT name, parent_id, real_size, id FROM file_entry where name like :name order by name limit :p_size;";
const SELECT_COUNT_ALL: &str = "SELECT COUNT(id) FROM file_entry;";
const SELECT_ALL_FILES: &str = "SELECT * FROM file_entry;";
const SELECT_FILES_NEXT_PAGE: &str = "SELECT name, parent_id, real_size, id FROM file_entry where name like :name and (name, id) >= (:p_name, :p_id) order by name limit :p_size;";
const FILE_ENTRY_NAME_INDEX: &str = "CREATE INDEX IF NOT EXISTS file_entry_name ON file_entry(name, id);";

const FILE_PAGE_SIZE: u32 = 3000;

pub fn main() -> Connection {
    let conn = Connection::open("test.db").unwrap();
//    let conn = Connection::open_in_memory().unwrap();

    conn.query_row("PRAGMA encoding;", &[], |row| {
        let x: String = row.get(0);
        assert_eq!("UTF-8", x);
    }).unwrap();


    conn.execute(CREATE_DB, &[]).unwrap();
    conn.prepare_cached(INSERT_FILE).unwrap();
    conn.prepare_cached(UPDATE_FILE).unwrap();
    conn.prepare_cached(DELETE_FILE).unwrap();
    conn.prepare_cached(UPSERT_FILE).unwrap();
    conn.prepare_cached(COUNT_FILES).unwrap();
    conn.prepare_cached(SELECT_FILES).unwrap();
    conn.prepare_cached(SELECT_FILES_NEXT_PAGE).unwrap();
    conn
}

pub fn delete_file(tx: &Transaction, file_id: u32) {
    tx.execute_named(DELETE_FILE, &[
        (":id", &file_id)]).unwrap();
}

//pub fn upsert_file(tx: &Transaction, file: &FileEntry) {
//    tx.execute_named(UPSERT_FILE, &[
//        (":id", &file.id),
//        (":parent_id", &file.parent_id),
//        (":dos_flags", &file.dos_flags),
//        (":real_size", &file.real_size),
//        (":name", &file.name),
//        (":modified_date", &file.modified_date),
//        (":created_date", &file.created_date),
//    ]).unwrap();
//}

//pub fn update_file(tx: &Transaction, file: &FileEntry) {
//    tx.execute_named(UPDATE_FILE, &[
//        (":id", &file.id),
//        (":parent_id", &file.parent_id),
//        (":dos_flags", &file.dos_flags),
//        (":real_size", &file.real_size),
//        (":name", &file.name),
//        (":modified_date", &file.modified_date),
//        (":created_date", &file.created_date),
//    ]).unwrap();
//}

pub fn create_indices(con: &Connection) {
    con.execute(FILE_ENTRY_NAME_INDEX, &[]).unwrap();
}

pub fn insert_files(connection: &mut Connection, files: &[FileEntry]) {
    let tx = connection.transaction().unwrap();
    {
        let mut stmt = tx.prepare_cached(INSERT_FILE).unwrap();
        for file in files {
            &file.names.iter().filter(|n| n.namespace != 2).for_each(|name| {
                stmt.execute_named(&[
                    (":id", &file.id),
                    (":parent_id", &name.parent_id),
                    (":dos_flags", &name.dos_flags),
                    (":real_size", &file.real_size),
                    (":name", &name.name),
                    (":modified_date", &file.modified_date),
                    (":created_date", &file.created_date),
                    (":base_record", &file.base_record),
                    (":fr_number", &file.fr_number),
                    (":namespace", &name.namespace),
                    (":flags", &file.flags)]).unwrap();
            });
        }
    }
    tx.commit().unwrap();
}

pub fn count_files(con: &Connection, name: &str) -> u32 {
    let mut statement = con.prepare_cached(COUNT_FILES).unwrap();
    let handle_row = |row: &Row| -> Result<u32> { Ok(row.get(0)) };
    let mut result = statement.query_and_then_named(&[(":name", &name)], handle_row).unwrap();
    result.nth(0).unwrap().unwrap()
}

pub fn load_all_arena() -> Result<(Arena)> {
    let con = Connection::open("test.db").unwrap();
    let count = con.query_row(SELECT_COUNT_ALL, &[], |r| r.get::<i32, u32>(0) as usize).unwrap();
    let mut stmt = con.prepare(SELECT_ALL_FILES).unwrap();
    let result = stmt.query_map(&[], FileEntity::from_file_row).unwrap();
    let mut arena = Arena::new(count);
    for file in result {
        let f: FileEntity = file??;
        arena.add_file(f);
    }
    Ok(arena)
}

