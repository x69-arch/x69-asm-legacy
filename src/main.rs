mod codegen;
mod instruction;
mod lexer;
mod parser;

use clap::{AppSettings, App, Arg};
use parser::{Log, ParseOptions, parse_file};
use codegen::assemble_lines;

use std::io::Write;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process;

fn print_logs_abort(logs: &[Log]) {
    let mut fatal = false;
    for log in logs {
        eprintln!("{}", log);
        fatal |= log.is_error();
    }
    if fatal {
        eprintln!("Aborting due to previous errors...");
        process::exit(1);
    }
}
fn make_log_and_abort(message: String, origin: &Path) -> ! {
    print_logs_abort(&[Log::IOError(message, origin.to_owned().into_os_string().into_string().unwrap())]);
    process::exit(1)
}

fn main() {
    let color = if cfg!(feature = "no_color") {
        AppSettings::ColorNever
    } else {
        AppSettings::ColorAuto
    };
    
    let arg_parse = App::new("Assembler")
        .about("The official x69 assembler!")
        .version(format!("v{}",env!("CARGO_PKG_VERSION")).as_str())
        .setting(color)
        .arg(Arg::new("FILE")
            // .required(true)
            .required_unless_present("list")
            .about("Input file to be assembled")
            .takes_value(true))
        .arg(Arg::new("output")
            .short('o')
            .long("output")
            .value_name("FILE")
            .takes_value(true))
        .arg(Arg::new("list")
            .about("Lists all available instructions")
            .long("list"))
        .get_matches();
    
    if arg_parse.is_present("list") {
        instruction::print_all();
        return;
    }
    
    let file_name = Path::new(arg_parse.value_of("FILE").unwrap());
    
    let parse_options = ParseOptions {
        origin: file_name.to_owned(),
        include_paths: vec![]
    };
    
    let (lines, logs) = parse_file(&parse_options);
    print_logs_abort(&logs);
    
    let (asm, logs) = assemble_lines(&lines);
    print_logs_abort(&logs);
    
    let output_name = arg_parse.value_of("output").map(PathBuf::from).unwrap_or_else(|| file_name.with_extension("o"));
    let mut output = match File::create(&output_name) {
        Ok(file) => file,
        Err(err) => make_log_and_abort(err.to_string(), &output_name),
    };
    if let Err(err) = output.write_all(&asm) {
        make_log_and_abort(err.to_string(), &output_name);
    }
}
