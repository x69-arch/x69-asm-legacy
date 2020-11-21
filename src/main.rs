extern crate utils;

mod codegen;
mod instruction;
mod lexer;
mod parser;

use std::fs::File;
use std::io::{Read, Write};

fn usage() {
    println!("The official x69 assembler!");
    println!("Usage: <input_file> [-o <output_file>]");
    println!("Help:  --help to see this message")
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut output_file = None;
    
    if let Some(output) = args.get(2) {
        if output == "-o" {
            if let Some(output) = args.get(3) {
                output_file = Some(output.clone());
            } else {
                usage();
                return;
            }
        } else {
            usage();
            return;
        }
    }
    
    if let Some(input_file) = args.get(1) {
        if input_file == "--help" {
            usage();
        } else {
            // Load file and stuff
            let file = File::open(input_file);
            match file {
                Ok(mut file) => {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).expect("Failed to read from file");
                    
                    let output_file = match output_file {
                        Some(file_name) => file_name,
                        None => {
                            let path = std::path::PathBuf::from(input_file);
                            path.with_extension("o").to_str().unwrap().to_owned()
                        }
                    };
                    
                    // Code assembling
                    // Code parsing
                    let (lines, mut logs) = parser::parse(&contents);
                    let assembly = codegen::assemble_lines(&lines, &mut logs);
                    
                    if !logs.is_empty() {
                        println!("{} message{} generated:", logs.len(), match logs.len() { 1 => "", _ => "s"});
                        let mut error = false;
                        for log in logs {
                            println!("{}", log);
                            error |= log.is_error();
                        }
                        if error {
                            println!("Aborting due to previous errors...");
                            return;
                        }
                    }
                    
                    let output = File::create(&output_file);
                    match output {
                        Ok(mut output) => {
                            output.write_all(assembly.as_slice()).expect("Failed to write binary to file");
                        },
                        
                        Err(err) => {
                            println!("Could not open file: \"{}\" for writing. {}", output_file, err);
                        }
                    }
                },
                
                Err(err) => {
                    println!("Could not open file: \"{}\" for reading. {}", input_file, err);
                }
            }
            
        }
    } else {
        usage();
    }
}
