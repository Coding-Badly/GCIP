use std::fmt::Display;
use std::fs;
use std::io::prelude::*;

use toml;

fn print_option<T: Display>(value: &Option<T>) {
    if let Some(v) = value {
        println!("{}", v);
    }
    else {
        println!("None");
    }
}

enum IndexOption <'a> {
    Integer(usize),
    Name(&'a str),
}

struct TomlValueByPath <'a> {
    path: Vec<IndexOption<'a>>,
}

impl <'a> TomlValueByPath <'a> {
    fn new() -> Self {
        Self {
            path: Vec::<IndexOption>::new(),
        }
    }
    fn add_name(&mut self, name: &'a str) -> &mut Self {
        self.path.push(IndexOption::Name(name));
        self
    }
    fn get(&self, root: &'a toml::Value) -> Option<&'a toml::Value> {
        let mut rover = root;
        for index in self.path.iter() {
            let possible_value = match index {
                IndexOption::Integer(v) => rover.get(v),
                IndexOption::Name(v) => rover.get(v),
            };
            if let Some(value) = possible_value {
                rover = value
            }
            else {
                return None;
            }
        }
        Some(rover)
    }
}

#[derive(Debug)]
struct VersionResourceInfo {
    company_name: Option<String>,
    legal_copyright: Option<String>,
    comments: Option<String>,
    product_name: Option<String>,
    file_description: Option<String>,
    file_version: Option<String>,
    product_version: Option<String>,
    original_filename: Option<String>,
    internal_name: Option<String>,
}

impl VersionResourceInfo {
    fn new() -> Self {
        Self {
            company_name: None,
            legal_copyright: None,
            comments: None,
            product_name: None,
            file_description: None,
            file_version: None,
            product_version: None,
            original_filename: None,
            internal_name: None,
        }
    }
}

/*
fn get_toml_value_by_path<'a>(root: &'a toml::Value, index_path: Vec<IndexOption>) -> Option<&'a toml::Value> {
    let mut rover = root;
    for index in index_path {
        let possible_value = match index {
            IndexOption::Integer(v) => rover.get(v),
            IndexOption::Name(v) => rover.get(v),
        };
        if let Some(value) = possible_value {
            rover = value
        }
        else {
            return None;
        }
    }
    Some(rover)
}
*/

fn print_value(name: &str, value: Option<&toml::Value>) {
    if let Some(v) = value {
        println!("{}: {}...", name, v.type_str());
        println!("{}", v);
    }
    else {
        println!("{}: None", name);
    }
}

fn output_error(message: &str) {
    eprintln!("{}", message);
    println!("cargo:warning={}", message);
}

fn read_cargo_toml() -> toml::Value {
    let mut contents = String::new();
    match fs::File::open("Cargo.toml") {
        Ok(mut inf) => {
            match inf.read_to_string(&mut contents) {
                Ok(_) => {},
                Err(error) =>  output_error(&format!("An error occurred trying to read the Cargo.toml file: {}", error)),
            }
        },
        Err(error) => output_error(&format!("An error occurred trying to open the Cargo.toml file: {}", error)),
    }
    match toml::from_str(&contents) {
        Ok(rv) => return rv,
        Err(error) => output_error(&format!("An error occurred trying to parse the Cargo.toml file: {}", error)),
    }
    toml::from_str("").unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
/*
https://doc.rust-lang.org/nightly/cargo/reference/build-script-examples.html
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("hello.rs");
    println!("cargo:rerun-if-changed=src/hello.c");
    warning=MESSAGE
*/
    let mut version_resource_info = VersionResourceInfo::new();

    let cargo_toml = read_cargo_toml();

//    const DEFAULT_COMPANY_NAME: Option<&str> = Some("Rowdy Dog Software");
//    DEFAULT_COMPANY_NAME.map(|v| v.to_string());

    if let Some(root) = TomlValueByPath::new()
            .add_name("package")
            .add_name("metadata")
            .add_name("version")
            .get(&cargo_toml) {
        version_resource_info.company_name = root.get("company-name").map_or(None, |v| v.as_str().map(|v| v.to_string()));
        version_resource_info.legal_copyright = root.get("legal-copyright").map_or(None, |v| v.as_str().map(|v| v.to_string()));
        version_resource_info.comments = root.get("comments").map_or(None, |v| v.as_str().map(|v| v.to_string()));
        version_resource_info.product_name = root.get("product-name").map_or(None, |v| v.as_str().map(|v| v.to_string()));
/*
    file_description: Option<String>,
    file_version: Option<String>,
    product_version: Option<String>,
    original_filename: Option<String>,
    internal_name: Option<String>,
*/
    }
    println!("{:?}", version_resource_info);

/*
    println!("{}", cargo_toml.type_str());  // table
    if let toml::Value::Table(ref t) = cargo_toml {
        println!("{:?}", t);
    }
*/
/*
    let possible_value = get_toml_value_by_path(&cargo_toml, vec![]);
    print_value("root", possible_value);
*/
/*
    let possible_value = TomlValueByPath::new()
        .add_name("package")
        .get(&cargo_toml);
    print_value("package", possible_value);
*/
/*
    let possible_value = get_toml_value_by_path(&cargo_toml, vec![IndexOption::Name("package")]);
    print_value("package", possible_value);
*/
/*
    if let Some(value) = possible_value {
        let possible_value = get_toml_value_by_path(&value, vec![IndexOption::Name("version")]);
        print_value("version", possible_value);
    }
*/
/*
[package.metadata.windows]
auto-generate-version-resource = true
*/
/*
    let value = TomlValueByPath::new()
        .add_name("package")
        .add_name("metadata")
        .add_name("windows")
        .add_name("auto-generate-version-resource")
        .get(&cargo_toml)
        .map_or(false, |v| v.as_bool().unwrap_or(false));
    println!("auto-generate-version-resource = {}", value);
    //print_value("auto-generate-version-resource", possible_value);
*/
/*
    let possible_value = get_toml_value_by_path(&cargo_toml, vec![IndexOption::Name("package"), IndexOption::Name("metadata")]);
    print_value("package", possible_value);
*/

    //print_option(&cargo_toml.as_str());
    //print_option(&cargo_toml["package"]["name"].as_str());

    Ok(())
}
