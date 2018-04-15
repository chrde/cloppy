extern crate embed_resource;
extern crate regex;

use std::io;
use std::fs::{
    File,
    OpenOptions,
};
use std::io::prelude::*;
use std::io::BufReader;
use regex::Regex;

fn main() {
    try_main().expect("Failed to build");
}

fn try_main() -> io::Result<()> {
    let output_path = "src/resources/__temp_resources.rc";
    let mut output = OpenOptions::new().create(true).write(true).truncate(true).open(output_path)?;
    add_constants(&mut output)?;
    add_declarations(&mut output)?;
    embed_resource::compile(output_path);
    Ok(())
}

fn add_declarations(output: &mut File) -> io::Result<()>{
    let mut resource = File::open("src/resources/resources.rc")?;
    let mut contents = String::new();
    resource.read_to_string(&mut contents)?;
    output.write_all(contents.as_bytes())?;
    Ok(())
}

fn add_constants(output: &mut File) -> io::Result<()>{
    let re = Regex::new(r"^pub const (?P<name>.*?): .*?= (?P<value>.*?);$").unwrap();
    let f = File::open("src/resources/constants.rs")?;
    for line in BufReader::new(f).lines() {
        let l = line?;
        assert!(re.is_match(&l));
        writeln!(output, "{}", re.replace_all(&l, "#define $name $value")).unwrap();
    }
    Ok(())
}
