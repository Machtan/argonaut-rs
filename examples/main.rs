extern crate argonaut;

use std::env;
use std::iter;
use argonaut::{ArgDef, parse, ParseError, help_arg, version_arg};
use std::process;

fn main() {
    if let Some(exit_code) = argonaut_main() {
        process::exit(exit_code);
    }
}

fn argonaut_main() -> Option<i32> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    
    let mut first = String::new();
    let mut second = String::new();
    let mut third = false;
    let mut cool_thing: Option<String> = None;
    let mut stars = 0;
    let mut verbose = false;
    let mut numbers: Vec<i32> = Vec::new();
    
    let description = "
        Test program for an argument parsing library.
        
        Showcases the features of the library.
    ";
    
    match parse("argonaut", &args, vec![
        ArgDef::pos("first", &mut first)
            .help("The first argument."),
        ArgDef::pos("second", &mut second)
            .help("The second argument."),
        ArgDef::pos("third-is-better", &mut third)
            .help("Whether the third argument is better than the rest."),
        ArgDef::trail("numbers", true, &mut numbers)
            .help("A bunch of numbers used for nefarious machinations."),
        
        ArgDef::option("cool", &mut cool_thing)
            .help("Something that you think is cool enough to pass."),
        ArgDef::count("star", &mut stars).short("s")
            .help("How many stars does this library deserve?"),
        ArgDef::flag("verbose", &mut verbose).short("v")
            .help("Print as much information as possible."),
        
        help_arg(description).short("h"),
        version_arg(),
    ]) {
        Ok(_) => {},
        Err(ParseError::Interrupted(_)) => {},
        Err(_) => return Some(1),
    };
    
    println!("First:   {}", first);
    println!("Second:  {}", second);
    println!("Third is better?:   {}", third);
    println!("");
    println!("Numbers: {:?}", numbers);
    if verbose {
        println!("VERBOSE!");
    }
    if let Some(cool) = cool_thing {
        println!("Got a real cool {} :u!", cool);
    } else {
        println!("Nothing's cool anymore");
    }
    println!("Library rating: {}", iter::repeat('*').take(stars).collect::<String>());
    
    None
}
