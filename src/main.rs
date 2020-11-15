mod codegen;
mod lexer;

use std::fs::File;
use std::io::{Read, Write};

fn usage() {
    println!("The official x69 assembler!");
    println!("Usage: <input_file> [-o <output_file>]");
    println!("Help:  --help to see this message")
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mut output_file = "a.out".to_owned();
    
    if let Some(output) = args.get(2) {
        if output == "-o" {
            if let Some(output) = args.get(3) {
                output_file = output.clone();
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
                    
                    let mut assembly = Vec::with_capacity(contents.len() >> 4);
                    
                    let err = codegen::assemble(contents.as_str(), &mut assembly);
                    if let Err(err) = err {
                        println!("Parsing error: {}", err);
                    }
                    
                    let output = File::create(output_file.clone());
                    match output {
                        Ok(mut output) => {
                            output.write_all(assembly.as_slice()).expect("Failed to write assembly to file");
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
