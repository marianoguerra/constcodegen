use anyhow::{anyhow, Result};
use toml::{self, Value};

fn main() -> Result<()> {
    let cmd = clap::Command::new("constcodegen")
        .bin_name("constcodegen")
        .subcommand_required(true)
        .subcommand(
            clap::Command::new("generate")
                .arg(
                    clap::Arg::new("language")
                        .long("language")
                        .value_parser(clap::builder::NonEmptyStringValueParser::new())
                        .action(clap::ArgAction::Set)
                        .default_value("js"),
                )
                .arg(
                    clap::Arg::new("root-dir")
                        .long("root-dir")
                        .value_parser(clap::builder::NonEmptyStringValueParser::new())
                        .action(clap::ArgAction::Set)
                        .default_value("."),
                )
                .arg(clap::Arg::new("input").required(true)),
        );
    let matches = cmd.get_matches();
    match matches.subcommand() {
        Some(("generate", matches)) => {
            let input = matches
                .get_one::<String>("input")
                .ok_or_else(|| anyhow!("input file is required"))?;

            match matches.get_one::<String>("language") {
                Some(v) if v == "rust-single" => {
                    generate(input, &mut RustOneFile::new(Box::new(std::io::stdout())))
                }
                Some(v) if v == "rust" => {
                    let root_path = create_and_get_root_path(matches)?;
                    generate(input, &mut RustMultiFile::new(root_path))
                }
                Some(v) if v == "js" => {
                    let root_path = create_and_get_root_path(matches)?;
                    generate(input, &mut JsMultiFile::new(root_path))
                }
                Some(v) if v == "js-single" => {
                    generate(input, &mut JsOneFile::new(Box::new(std::io::stdout())))
                }
                _ => Err(anyhow!("unknown language")),
            }
        }

        _ => Err(anyhow!("subcommand not found")),
    }
}

fn create_and_get_root_path(matches: &clap::ArgMatches) -> Result<std::path::PathBuf> {
    let default_raw_path = ".".into();
    let raw_path = matches
        .get_one::<String>("root-dir")
        .unwrap_or(&default_raw_path);
    let root_path = std::path::Path::new(raw_path).to_path_buf();
    std::fs::create_dir_all(root_path.clone())?;
    Ok(root_path)
}

trait CodeGen {
    fn generate(&mut self, name: &str, doc: &toml::value::Table) -> Result<()>;
}

struct RustOneFile {
    writer: Box<dyn std::io::Write>,
}

impl RustOneFile {
    pub fn new(writer: Box<dyn std::io::Write>) -> Self {
        Self { writer }
    }
}

impl CodeGen for RustOneFile {
    fn generate(&mut self, name: &str, doc: &toml::value::Table) -> Result<()> {
        writeln!(self.writer, "mod {} {{", name)?;
        generate_rust_mod(doc, "    ", &mut *self.writer)?;
        writeln!(self.writer, "}}\n")?;
        Ok(())
    }
}

struct JsOneFile {
    writer: Box<dyn std::io::Write>,
}

impl JsOneFile {
    pub fn new(writer: Box<dyn std::io::Write>) -> Self {
        Self { writer }
    }
}

impl CodeGen for JsOneFile {
    fn generate(&mut self, name: &str, doc: &toml::value::Table) -> Result<()> {
        writeln!(self.writer, "// {}", name)?;
        let prefix = format!("{}_", name.to_string().to_uppercase());
        generate_js_mod(doc, &prefix, &mut *self.writer)?;
        writeln!(self.writer, "\n")?;
        Ok(())
    }
}

struct RustMultiFile {
    root_path: std::path::PathBuf,
}

impl RustMultiFile {
    pub fn new(root_path: std::path::PathBuf) -> Self {
        Self { root_path }
    }
}

impl CodeGen for RustMultiFile {
    fn generate(&mut self, name: &str, doc: &toml::value::Table) -> Result<()> {
        let path = self.root_path.join(format!("{}.rs", name));
        let mut writer = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)?;
        generate_rust_mod(doc, "", &mut writer)?;
        Ok(())
    }
}

struct JsMultiFile {
    root_path: std::path::PathBuf,
}

impl JsMultiFile {
    pub fn new(root_path: std::path::PathBuf) -> Self {
        Self { root_path }
    }
}

impl CodeGen for JsMultiFile {
    fn generate(&mut self, name: &str, doc: &toml::value::Table) -> Result<()> {
        let path = self.root_path.join(format!("{}.js", name));
        let mut writer = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)?;
        generate_js_mod(doc, "", &mut writer)?;
        Ok(())
    }
}

fn generate(path: &str, code_gen: &mut impl CodeGen) -> Result<()> {
    let toml_string = std::fs::read_to_string(path)?;
    let value: Value = toml::from_str(&toml_string)?;
    let doc = value
        .as_table()
        .ok_or_else(|| anyhow!("toml file is not a table"))?;
    for (ns, table) in doc {
        code_gen.generate(
            ns,
            table
                .as_table()
                .ok_or_else(|| anyhow!("toml namespace is not a table"))?,
        )?;
    }

    Ok(())
}

fn generate_js_mod(
    table: &toml::value::Table,
    prefix: &str,
    writer: &mut dyn std::io::Write,
) -> Result<()> {
    for (name, value) in table {
        writeln!(writer, "export const {}{} = {};", prefix, name, value)?;
    }

    Ok(())
}

fn generate_rust_mod(
    table: &toml::value::Table,
    indent: &str,
    writer: &mut dyn std::io::Write,
) -> Result<()> {
    writeln!(writer, "#[allow(dead_code)]")?;
    for (name, value) in table {
        writeln!(
            writer,
            "{}pub const {}: {} = {};",
            indent,
            name,
            value_to_rust_type(value),
            value
        )?;
    }

    Ok(())
}

fn value_to_rust_type(v: &Value) -> String {
    match v {
        Value::String(_) => "&str",
        Value::Integer(_) => "i64",
        Value::Float(_) => "f64",
        Value::Boolean(_) => "bool",
        _ => "BadType",
    }
    .into()
}
