use ntfs::FileEntry;
use rusqlite::Connection;
use rusqlite::Result;
use rusqlite::Row;
use rusqlite::Transaction;
use windows::utils::ToWide;
use rusqlite::types::ToSql;

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
const SELECT_FILES_NEXT_PAGE: &str = "SELECT name, parent_id, real_size, id FROM file_entry where name like :name and (name, id) > (:p_name, :p_id) order by name limit :p_size;";
const FILE_ENTRY_NAME_INDEX: &str = "CREATE INDEX IF NOT EXISTS file_entry_name ON file_entry(name, id);";

const FILE_PAGE_SIZE: usize = 300;

pub fn main() -> Connection {
    let conn = Connection::open("test.db").unwrap();
//    let conn = Connection::open_in_memory().unwrap();

    conn.query_row("PRAGMA encoding;", &[], |row| {
        let x: String = row.get(0);
        println!("{}", x);
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

pub fn count_files(con: &Connection, name: &String) -> u32 {
    let mut statement = con.prepare_cached(COUNT_FILES).unwrap();
    let handle_row = |row: &Row| -> Result<u32> { Ok(row.get(0)) };
    let mut result = statement.query_and_then_named(&[(":name", name)], handle_row).unwrap();
    result.nth(0).unwrap().unwrap()
}

pub struct FileNextPage {
    file_id: u32,
    file_name: String,
}

fn paginate_results(mut rows: Vec<FileEntity>) -> (Vec<FileEntity>, Option<FileNextPage>) {
    let page = if rows.len() > FILE_PAGE_SIZE {
        assert_eq!(FILE_PAGE_SIZE + 1, rows.len());
        let last = rows.pop().unwrap();
        Some(FileNextPage { file_id: last.id, file_name: last.name })
    } else {
        None
    };
    (rows, page)
}

pub struct FileEntity {
    name: String,
    name_wide: Vec<u16>,
    path: Vec<u16>,
    size: Vec<u16>,
    id: u32,
}

impl FileEntity {
    pub fn from_file_row(row: &Row) -> Result<Self> {
        use windows::utils::ToWide;
        let name = row.get::<i32, String>(0);
        let name_wide = name.to_wide_null();
        let path = row.get::<i32, i64>(1).to_string().to_wide_null();
        let size = row.get::<i32, i64>(2).to_string().to_wide_null();
        let id = row.get::<i32, u32>(2);
        Ok(FileEntity { name, name_wide, path, size, id })
    }

    pub fn name_wide(&self) -> &[u16] {
        &self.name_wide
    }

    pub fn path(&self) -> &[u16] {
        &self.path
    }

    pub fn size(&self) -> &[u16] {
        &self.size
    }
}

pub fn select_files_params<'a>(name: &'a String, page: Option<&'a FileNextPage>) -> (&'static str, Vec<(&'a str, &'a ToSql)>) {
    let mut params: Vec<(&str, &ToSql)> = Vec::new();
    let query = match page {
        Some(p) => {
            params.push((":name", name));
            params.push((":p_size", &(FILE_PAGE_SIZE as u32 + 1)));
            params.push((":p_name", &p.file_name));
            params.push((":p_id", &p.file_id));
            SELECT_FILES_NEXT_PAGE
        }
        None => {
            params.push((":name", name));
            params.push((":p_size", &(FILE_PAGE_SIZE as u32 + 1)));
            SELECT_FILES
        }
    };
    (query, params)
}

pub fn select_files(con: &Connection, name: &String, page: Option<FileNextPage>) -> Result<(Vec<FileEntity>, Option<FileNextPage>)> {
    let (query, params) = select_files_params(name, page.as_ref());
    let mut statement = con.prepare_cached(query).unwrap();
    let result = statement.query_and_then_named(&params, FileEntity::from_file_row).unwrap();
    let mut entries = Vec::new();
    for entry in result {
        entries.push(entry?);
    }
    Ok(paginate_results(entries))
}
