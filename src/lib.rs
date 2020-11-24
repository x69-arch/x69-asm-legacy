mod codegen;
mod instruction;
mod lexer;
mod parser;

extern crate libc;

use libc::c_char;
use std::ffi::CStr;
use std::fs::File;
use std::io::{Read, Write};

///# Safety
/// None
#[no_mangle]
pub unsafe extern "C" fn assemble_x69(input_name: *const c_char, output_file: *const c_char) {
    let input_path = CStr::from_ptr(input_name).to_str().unwrap();
    let output_path = CStr::from_ptr(output_file).to_str().unwrap();
    
    let mut input = File::open(input_path).unwrap();
    let mut output = File::create(output_path).unwrap();
    
    let mut contents = String::new();
    input.read_to_string(&mut contents).unwrap();
    
    let (lines, mut logs) = parser::parse(&contents);
    let binary = codegen::assemble_lines(&lines, &mut logs);
    
    for log in logs {
        eprintln!("{}", log);
    }
    
    output.write_all(&binary).unwrap();
}
