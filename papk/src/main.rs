extern crate abxml;
extern crate zip;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

use std::io::{BufReader, Read, Cursor};
use abxml::visitor::{ModelVisitor, Executor, XmlVisitor, Resources};
use abxml::encoder::Xml;
use failure::{bail, Error, ResultExt};

fn main() {
    println!("Hello, world!");
}

pub fn parse_apk(apk_path: String, target_file: String) -> Result<(), Error> {
    let android_resources_content = abxml::STR_ARSC.to_owned();

    // APK
    let file = std::fs::File::open(&apk_path)?;
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut resources_content = Vec::new();
    archive
        .by_name("resources.arsc")
        .unwrap()
        .read_to_end(&mut resources_content)?;

    let mut resources_visitor = ModelVisitor::default();
    Executor::arsc(&resources_content, &mut resources_visitor)?;
    Executor::arsc(&android_resources_content, &mut resources_visitor)?;

    for i in 0..archive.len() {
        let mut current_file = archive.by_index(i).unwrap();

        if current_file.name().contains(&target_file) {
            {
                println!("Current file: {}", current_file.name());
                let mut xml_content = Vec::new();
                current_file.read_to_end(&mut xml_content)?;
                let new_content = xml_content.clone();

                let resources = resources_visitor.get_resources();
                let out =
                    parse_xml(&new_content, resources).context("could not decode target file")?;
                println!("{}", out);
            }
        }
    }

    Ok(())
}

fn parse_xml<'a>(content: &[u8], resources: &'a Resources<'a>) -> Result<String, Error> {
    let cursor = Cursor::new(content);
    let mut visitor = XmlVisitor::new(resources);

    Executor::xml(cursor, &mut visitor)?;

    match *visitor.get_root() {
        Some(ref root) => match *visitor.get_string_table() {
            Some(_) => {
                let res =
                    Xml::encode(visitor.get_namespaces(), root).context("could note encode XML")?;
                return Ok(res);
            }
            None => {
                println!("No string table found");
            }
        },
        None => {
            println!("No root on target XML");
        }
    }

    bail!("could not decode XML")
}

#[cfg(test)]
mod tests {
    use crate::parse_apk;
    use failure::Error;

    #[test]
    fn test_parse_apk_binary() {
        let result = parse_apk(String::from("../_fixtures/apk/app-release-unsigned.apk"),
                               String::from("standalone_badge.xml"));
        match result {
            Ok(_) => {
                println!("zz")
            }
            Err(_) => {
                println!("error")
            }
        }
    }
}