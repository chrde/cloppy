use ini::Ini;

pub struct UserSettings {

}

impl UserSettings {
    pub fn new() {
        let mut conf = Ini::new();
        conf.with_section(None::<String>)
            .set("encoding", "utf-8");
        conf.with_section(Some("User".to_owned()))
            .set("given_name", "Tommy")
            .set("family_name", "Green")
            .set("unicode", "Raspberry树莓");
        conf.with_section(Some("Book".to_owned()))
            .set("name", "Rust cool");
        conf.write_to_file("conf.ini").unwrap();
    }
}