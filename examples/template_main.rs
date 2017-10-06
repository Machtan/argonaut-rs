extern crate argonaut;

use std::env;
use std::iter;
use argonaut::{ArgDef, parse, ParseError};
use std::process;
use std::collections::HashSet;

fn main() {
    // Properly set exit codes after the program has cleaned up.
    if let Some(exit_code) = argonaut_main() {
        process::exit(exit_code);
    }
}

fn argonaut_main() -> Option<i32> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    
    // Set variables
    let mut first = String::new();
    let mut second = String::new();
    let mut third = false;
    let mut cool_thing: Option<String> = None;
    let mut stars = 0;
    let mut verbose = false;
    let mut numbers: Vec<i32> = Vec::new();
    let mut includes: HashSet<String> = HashSet::new();

    let description = "
        Test program for an argument parsing library.
        
        Showcases the features of the library.
    ";
    
    // Declare what arguments are expected and how to parse them
    match parse("argonaut", &args, vec![
        ArgDef::positional("first", &mut first)
            .help("The first argument.")
        
        , ArgDef::positional("second", &mut second)
            .help("The second argument.")
            
        , ArgDef::positional("third-is-better", &mut third)
            .help("Whether the third argument is better than the rest.")
            
        , ArgDef::trail("numbers", true, &mut numbers)
            .help("A bunch of numbers used for nefarious machinations.")
        
        , ArgDef::collect("include", &mut includes)
            .short("i")
            .param("file")
            .help("Which files to include in the cake.")
        
        , ArgDef::setting("cool", &mut cool_thing).param("thing")
            .help("Something that you think is cool enough to pass.")
        
        , ArgDef::count("star", &mut stars).short("s")
            .help("How many stars does this library deserve?")
        
        , ArgDef::flag("verbose", &mut verbose).short("v")
            .help("Print as much information as possible.")
        
        , ArgDef::default_help(description).short("h")
        , ArgDef::default_version()
    ]) {
        Ok(_optional_error_code) => {},
        Err(ParseError::Interrupted(_)) => {
            return None;
        },
        Err(_) => {
            return Some(1);
        }
    };
    
    // Use the parsed arguments after a succesful parse
    println!("First:   {}", first);
    println!("Second:  {}", second);
    println!("Third is better?: {}", third);
    println!("");
    println!("Numbers:          {:?}", numbers);
    println!("Included files:   {:?}", includes);
    if verbose {
        println!("VERBOSE!");
    }
    if let Some(cool) = cool_thing {
        println!("Got a real cool {} :u!", cool);
    } else {
        println!("Nothing's cool anymore");
    }
    println!("Library rating: {}", iter::repeat('*').take(stars).collect::<String>());
    
    // Return no error code
    None
}
