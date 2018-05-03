use ntfs::FileEntry;
use rusqlite::Connection;
use rusqlite::Result;
use rusqlite::Row;
use rusqlite::Transaction;
use rusqlite::types::ToSql;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ops::Range;
use std::time::Instant;
use winapi::shared::ntdef::LPWSTR;
use std::collections::HashMap;

mod arena;
pub use self::arena::Arena;

const CREATE_DB: &str = "
    CREATE TABLE IF NOT EXISTS file_entry (
    id            INTEGER PRIMARY KEY,
    parent_id     INTEGER,
    dos_flags     INTEGER,
    real_size     INTEGER,
    logical_size  INTEGER,
    name          TEXT,
    modified_date INTEGER,
    created_date  INTEGER
    );";
const INSERT_FILE: &str = "INSERT INTO file_entry (id, parent_id, dos_flags, real_size, logical_size, name, modified_date, created_date) \
    VALUES (:id, :parent_id, :dos_flags, :real_size, :logical_size, :name, :modified_date, :created_date);";
const UPSERT_FILE: &str = "INSERT OR REPLACE INTO file_entry (id, parent_id, dos_flags, real_size, logical_size, name, modified_date, created_date) \
    VALUES (:id, :parent_id, :dos_flags, :real_size, :logical_size, :name, :modified_date, :created_date);";
const UPDATE_FILE: &str = "UPDATE file_entry SET \
    id = :id, parent_id = :parent_id, dos_flags = :dos_flags, real_size = :real_size, logical_size = :logical_size, name = :name, modified_date = :modified_date, created_date = :created_date \
    WHERE id = :id;";
const DELETE_FILE: &str = "DELETE FROM file_entry WHERE id = :id;";
const COUNT_FILES: &str = "SELECT COUNT(id) FROM file_entry where name like :name";
const SELECT_FILES: &str = "SELECT name, parent_id, real_size, id FROM file_entry where name like :name order by name limit :p_size;";
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

pub fn upsert_file(tx: &Transaction, file: &FileEntry) {
    tx.execute_named(UPSERT_FILE, &[
        (":id", &file.id),
        (":parent_id", &file.parent_id),
        (":dos_flags", &file.dos_flags),
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
        (":dos_flags", &file.dos_flags),
        (":real_size", &file.real_size),
        (":logical_size", &file.logical_size),
        (":name", &file.name),
        (":modified_date", &file.modified_date),
        (":created_date", &file.created_date),
    ]).unwrap();
}

pub fn create_indices(con: &Connection) {
    con.execute(FILE_ENTRY_NAME_INDEX, &[]).unwrap();
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
                    (":dos_flags", &file.dos_flags),
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

pub fn count_files(con: &Connection, name: &str) -> u32 {
    let mut statement = con.prepare_cached(COUNT_FILES).unwrap();
    let handle_row = |row: &Row| -> Result<u32> { Ok(row.get(0)) };
    let mut result = statement.query_and_then_named(&[(":name", &name)], handle_row).unwrap();
    result.nth(0).unwrap().unwrap()
}

#[derive(Debug)]
pub struct Page {
    file_id: u32,
    file_name: String,
    pub page_size: u32,
}

#[derive(Default, Debug)]
pub struct Query {
    query: String,
    page: Option<Page>,
}

impl Query {
    pub fn new(query: String) -> Self {
        Query {
            query,
            page: None,
        }
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn next(&self) -> Option<&Page> {
        self.page.as_ref()
    }
    pub fn has_more(&self) -> bool {
        return self.page.is_some();
    }
}

fn paginate_results(mut rows: Vec<FileEntity>, query: String) -> (Vec<FileEntity>, Query) {
    let page = if rows.len() > FILE_PAGE_SIZE as usize {
        assert_eq!(FILE_PAGE_SIZE + 1, rows.len() as u32);
        let last = rows.pop().unwrap();
        Some(Page { file_id: last.id, file_name: last.name, page_size: FILE_PAGE_SIZE })
    } else {
        None
    };
    (rows, Query { query, page })
}

#[derive(Default, Clone, Eq)]
pub struct FileEntity {
    name: String,
    path: Vec<u8>,
    size: Vec<u8>,
    id: u32,
}

#[derive(Default, Clone, Eq)]
pub struct FileKey {
    name: Vec<u8>,
    id: u32,
    position: usize,
}

impl FileKey {
    pub fn new(name: String, id: u32) -> Self {
        use windows::utils::ToWide;
        let name = name.into_bytes();
        FileKey {
            name,
            id,
            position: 0,
        }
    }

    pub fn name_str(&self) -> String {
        use std::ffi::{OsStr, OsString};
        use std::os::windows::ffi::{OsStrExt, OsStringExt};
        use windows::utils::FromWide;
        String::from_utf8((&self.name).clone()).unwrap()
//        OsString::from_wide_null(&self.name[..]).to_string_lossy().to_string()
    }

    pub fn position(&self) -> usize {
        self.position
    }
}

impl FileEntity {
    pub fn new(name: String, id: u32) -> Self {
        FileEntity {
            name,
            id,
            ..Default::default()
        }
    }

    pub fn from_file_row(row: &Row) -> Result<Self> {
        use windows::utils::ToWide;
        let name = row.get::<i32, String>(5);
        let path = row.get::<i32, i64>(1).to_string().into_bytes();
        let size = row.get::<i32, i64>(3).to_string().into_bytes();
        let id = row.get::<i32, u32>(0);
        Ok(FileEntity { name, path, size, id })
    }

    pub fn path(&self) -> &[u8] {
        &self.path
    }

    pub fn size(&self) -> &[u8] {
        &self.size
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

fn select_files_params<'a>(name: &'a String, page: Option<&'a Page>) -> (&'static str, Vec<(&'a str, &'a ToSql)>) {
    let mut params: Vec<(&str, &ToSql)> = Vec::new();
    let query = match page {
        Some(p) => {
            params.push((":name", name));
            params.push((":p_size", &(FILE_PAGE_SIZE + 1)));
            params.push((":p_name", &p.file_name));
            params.push((":p_id", &p.file_id));
            SELECT_FILES_NEXT_PAGE
        }
        None => {
            params.push((":name", name));
            params.push((":p_size", &(FILE_PAGE_SIZE + 1)));
            SELECT_FILES
        }
    };
    (query, params)
}

pub fn select_files(con: &Connection, query: &Query) -> Result<(Vec<FileEntity>, Query)> {
    let (sql_query, params) = select_files_params(&query.query, query.page.as_ref());
    let mut statement = con.prepare_cached(sql_query).unwrap();
    let result = statement.query_and_then_named(&params, FileEntity::from_file_row).unwrap();
    let mut entries = Vec::new();
    for entry in result {
        entries.push(entry?);
    }
    Ok(paginate_results(entries, query.query.clone()))
}

pub fn insert_tree() -> Result<(Vec<FileKey>, Arena)> {
    let con = Connection::open("test.db").unwrap();
    let mut arena = Arena::new();
    let mut stmt = con.prepare(SELECT_ALL_FILES).unwrap();
    let result = stmt.query_map(&[], FileEntity::from_file_row).unwrap();
    let mut tree = BTreeSet::<FileKey>::new();
    for file in result {
        let f: FileEntity = file??;
        let key = arena.add_file(f.clone());
//        tree.insert(key);
    }

//    let files = tree.into_iter().collect::<Vec<FileKey>>();
//    println!("{} {} {}", files.len(), arena.files.len(), arena.data.len());
    ::std::thread::sleep_ms(5000);
    Ok((Vec::new(), Arena::new()))
}

pub fn insert_tree1() -> Result<(Arena)> {
    let con = Connection::open("test.db").unwrap();
    let mut stmt = con.prepare(SELECT_ALL_FILES).unwrap();
    let result = stmt.query_map(&[], FileEntity::from_file_row).unwrap();
    let mut arena = Arena::new();
    for file in result {
        let f: FileEntity = file??;
        let key = arena.add_file(f.clone());
    }

//    println!("{} {} {}", files.len(), arena.files.len(), arena.data.len());
    Ok(arena)
}

impl Ord for FileEntity {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.name, self.id).cmp(&(&other.name, other.id))
    }
}

impl PartialEq for FileEntity {
    fn eq(&self, other: &FileEntity) -> bool {
        (&self.name, self.id) == (&other.name, other.id)
    }
}

impl PartialOrd for FileEntity {
    fn partial_cmp(&self, other: &FileEntity) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for FileKey {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.name, self.id).cmp(&(&other.name, other.id))
    }
}

impl PartialEq for FileKey {
    fn eq(&self, other: &FileKey) -> bool {
        (&self.name, self.id) == (&other.name, other.id)
    }
}

impl PartialOrd for FileKey {
    fn partial_cmp(&self, other: &FileKey) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}