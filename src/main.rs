use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use kastel_markup::{render_html, render_html_document};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    if let Err(error) = run() {
        eprintln!("km: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let args: Vec<String> = env::args().skip(1).collect();
    let cli = Cli::parse(&args)?;

    if cli.help {
        print_help();
        return Ok(());
    }

    if cli.version {
        println!("km {VERSION}");
        return Ok(());
    }

    let source = read_input(&cli.input)?;

    let output = if cli.standalone {
        render_html_document(&source)
    } else {
        render_html(&source)
    }
    .map_err(|error| CliError(format!("{error}")))?;

    write_output(cli.output.as_deref(), &output)
}

#[derive(Debug)]
struct Cli {
    input: String,
    output: Option<String>,
    format: String,
    standalone: bool,
    help: bool,
    version: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            input: String::new(),
            output: None,
            format: "html".into(),
            standalone: false,
            help: false,
            version: false,
        }
    }
}

impl Cli {
    fn parse(args: &[String]) -> Result<Self, CliError> {
        let mut cli = Self::default();
        let mut index = 0;

        while index < args.len() {
            match args[index].as_str() {
                "-h" | "--help" => cli.help = true,
                "-V" | "--version" => cli.version = true,
                "--standalone" => cli.standalone = true,
                "-o" | "--output" => {
                    index += 1;
                    cli.output = Some(required_value(args, index, "--output")?);
                }
                "--format" => {
                    index += 1;
                    cli.format = required_value(args, index, "--format")?.to_ascii_lowercase();
                }
                value if value.starts_with("--format=") => {
                    let format = value.strip_prefix("--format=").unwrap();
                    if format.is_empty() {
                        return Err(CliError("missing value after --format=".into()));
                    }
                    cli.format = format.to_ascii_lowercase();
                }
                value if value.starts_with('-') && value != "-" => {
                    return Err(CliError(format!("unknown option `{value}`")));
                }
                value => {
                    if !cli.input.is_empty() {
                        return Err(CliError("multiple input files provided".into()));
                    }
                    cli.input = value.into();
                }
            }

            index += 1;
        }

        if cli.help || cli.version {
            return Ok(cli);
        }

        if cli.input.is_empty() {
            return Err(CliError("missing input file\n\nUsage: km [OPTIONS] <INPUT>".into()));
        }

        if cli.format != "html" {
            return Err(CliError(format!(
                "unsupported format `{}`; available formats: html",
                cli.format
            )));
        }

        Ok(cli)
    }
}

fn required_value(
    args: &[String],
    index: usize,
    option: &str,
) -> Result<String, CliError> {
    args.get(index)
        .cloned()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| CliError(format!("missing value after {option}")))
}

fn read_input(input: &str) -> Result<String, CliError> {
    if input == "-" {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .map_err(|error| CliError(format!("cannot read stdin: {error}")))?;
        return Ok(source);
    }

    fs::read_to_string(input)
        .map_err(|error| CliError(format!("cannot read `{input}`: {error}")))
}

fn write_output(output: Option<&str>, content: &str) -> Result<(), CliError> {
    match output {
        None | Some("-") => {
            print!("{content}");
            Ok(())
        }
        Some(path) => {
            let path = Path::new(path);

            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    fs::create_dir_all(parent).map_err(|error| {
                        CliError(format!(
                            "cannot create output directory `{}`: {error}",
                            parent.display()
                        ))
                    })?;
                }
            }

            fs::write(path, content)
                .map_err(|error| CliError(format!("cannot write `{}`: {error}", path.display())))
        }
    }
}

fn print_help() {
    println!(
        "Kastel Markup CLI\n\n\
Usage:\n  km [OPTIONS] <INPUT>\n\n\
Options:\n  -o, --output <FILE>   Write output to FILE\n      --format <FMT>    Output format (html)\n      --standalone      Generate a complete HTML5 document\n  -h, --help            Show this help\n  -V, --version         Show version\n\n\
Input:\n  <INPUT>               .km file\n  -                     Read from stdin\n\n\
Output:\n  No --output           Write to stdout\n  --output <FILE>       Write to FILE\n  --output -            Write to stdout\n\n\
Examples:\n  km document.km\n  km document.km -o document.html\n  km document.km --standalone -o document.html\n  km - < document.km\n"
    );
}

#[derive(Debug)]
struct CliError(String);

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CliError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_input() {
        let cli = Cli::parse(&["document.km".into()]).unwrap();
        assert_eq!(cli.input, "document.km");
        assert_eq!(cli.format, "html");
        assert!(!cli.standalone);
    }

    #[test]
    fn parses_output_and_standalone() {
        let cli = Cli::parse(&[
            "document.km".into(),
            "--standalone".into(),
            "--output".into(),
            "document.html".into(),
        ])
        .unwrap();

        assert!(cli.standalone);
        assert_eq!(cli.output.as_deref(), Some("document.html"));
    }

    #[test]
    fn parses_short_output() {
        let cli = Cli::parse(&["document.km".into(), "-o".into(), "output.html".into()]).unwrap();
        assert_eq!(cli.output.as_deref(), Some("output.html"));
    }

    #[test]
    fn parses_format() {
        let cli = Cli::parse(&["document.km".into(), "--format=html".into()]).unwrap();
        assert_eq!(cli.format, "html");
    }

    #[test]
    fn parses_help() {
        assert!(Cli::parse(&["--help".into()]).unwrap().help);
    }

    #[test]
    fn parses_version() {
        assert!(Cli::parse(&["--version".into()]).unwrap().version);
    }

    #[test]
    fn accepts_stdin() {
        assert_eq!(Cli::parse(&["-".into()]).unwrap().input, "-");
    }

    #[test]
    fn rejects_unknown_option() {
        assert!(Cli::parse(&["--unknown".into()]).is_err());
    }

    #[test]
    fn rejects_multiple_inputs() {
        assert!(Cli::parse(&["one.km".into(), "two.km".into()]).is_err());
    }

    #[test]
    fn rejects_unsupported_format() {
        assert!(Cli::parse(&["document.km".into(), "--format".into(), "terminal".into()]).is_err());
    }
}
