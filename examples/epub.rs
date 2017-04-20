extern crate argonaut;

use std::env;
use argonaut::{ArgDef, parse, ParseError, help_arg, version_arg};
use std::process;

fn main() {
    if let Some(exit_code) = epub_main() {
        process::exit(exit_code);
    }
}

fn create_epub(spec_file: &str, target_path: Option<String>, is_raw_spec: bool) {
    println!("Creating epub from spec: '{}' (target_path: {:?}, is raw spec?: {})", 
        spec_file, target_path, is_raw_spec);
}

fn print_spec_template() {
    println!("<Template goes here>");
}

fn create_epub_from_folder(folder: &str) {
    println!("Creating epub from image folder '{}'...", folder);
}

fn epub_main() -> Option<i32> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let description = "
        Program to create ePub e-book files.
    ";
    
    match parse("epub", &args, vec![
        // Defines a subcommand named 'create' that will run the given closure
        // or function when it is encountered during a parse.
        
        // The 'program' argument is a String with the name of the command that
        // is currently running, in this case "epub create".
        ArgDef::subcommand("create", |program, args| {
            let mut spec_file = String::new();
            let mut target_path: Option<String> = None;
            let mut is_raw_spec = false;
            
            parse(program, args, vec![
                ArgDef::positional("spec_file", &mut spec_file)
                    .help("The TOML specification of the book"),
                
                ArgDef::setting("target_path", &mut target_path).short("t")
                    .help("
                        A specific path to compile the ePub to. Defaults to a
                        name/author coupling in the current working directory
                    "),
                
                ArgDef::flag("is_raw_spec", &mut is_raw_spec).short("r")
                    .help("
                        Interpret the spec-file argument as the contents of the 
                        specification file, instead of a path to it.
                    "),
                
                // This creates a simple 'interrupt' flag for '--help' that
                // shows help for this subcommand with the given description.
                help_arg("
                    Compiles an ePub from a markdown source and a TOML specification. The files in
                    the specification are sought relatively to the location of the specification
                    file, so use absolute paths when needed. If no arguments are given, the
                    created file will be found in the active working directory.
                ") 
                // The common short form '-h' is not bound by default.
                .short("h"), 
            ])?; 
            // The subcommand closure returns the same type as 'parse', so this
            // is the expected way to handle subparsing.
            
            // The parse has succeeded and all required values assigned.
            create_epub(&spec_file, target_path, is_raw_spec);
            
            // In this simple example, no error code is returned.
            
            // But if create_epub for instance checked whether the folder existed
            // an error code for that might be passed here, so that 'main' could
            // call process::exit with it later.
            Ok(None)
        })
        .help("Creates a new ePub from a given specification."),
        
        ArgDef::subcommand("example", |program, args| {
            parse(program, args, vec![])?;
            
            print_spec_template();
            
            Ok(None)
        })
        .help("Prints a template for an ePub specification file."),
        
        ArgDef::subcommand("from_folder", |program, args| {
            let mut folder = String::new();
            
            parse(program, args, vec![
                ArgDef::positional("folder", &mut folder)
                    .help("The folder to load images from."),
                
                help_arg(
                "
                    Creates a simple epub from the images in the given folder.
                    This is useful for quickly creating rather bad comic epubs.
                "),
            ])?;
            
            create_epub_from_folder(&folder);
            
            Ok(None)
        })
        .help("Creates a simple ePub from the images in the given folder."),
        
        help_arg(description).short("h"),
        version_arg(),
    ]) 
    // This is probably how any subcommand-parsing function should look
    // Ok and Interrupt means that it terminated as expected.
    // Otherwise return an error code to be set with process::exit.
    {
        Ok(_) => None,
        Err(ParseError::Interrupted(_)) => None,
        Err(_) => Some(1),
    }
}
