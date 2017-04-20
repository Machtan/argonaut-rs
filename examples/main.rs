extern crate argonaut;

use std::env;
use std::iter;
use argonaut::{ArgDef, parse, ParseError, help_arg, version_arg};
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
    
    // Set up the variables that will be parsed into.
    
    let mut first = String::new();
    let mut second = String::new();
    let mut third = false;
    
    // Types that are FromStr + Debug can be set by the parser, so here
    // we need to tell it that we want a String.
    let mut cool_thing: Option<String> = None;
    let mut stars = 0;
    let mut verbose = false;
    
    // And here we need to tell what elements should be collected into the collection.
    let mut numbers: Vec<i32> = Vec::new();
    
    // All set/linear collections in std::collections are supported.
    // 
    // To add more targets, implement CollectionTarget for your collection type.
    let mut includes: HashSet<String> = HashSet::new();
    
    // Prepare a description of the program.
    // 
    // The default help formatter removes initial/final empty lines, and
    // strips the lines, so the text can be written with convenient indentation.
    let description = "
        Test program for an argument parsing library.
        
        Showcases the features of the library.
    ";
    
    // Starts parsing arguments with the program name 'argonaut', and the
    // definitions in the vector.
    // 
    // If the definitions are invalid (both subcommands and positional arguments
    // defined, names used twice, etc.), the function will PANIC!.
    match parse("argonaut", &args, vec![
        // Declare a positional argument with the name 'first', that should
        // store its value in the String variable 'first'.
        ArgDef::positional("first", &mut first)
            // This describes the argument in the generated help messages.
            .help("The first argument."),
        
        ArgDef::positional("second", &mut second)
            .help("The second argument."),
        
        // Declare a positional argument that will try to parse its value
        // as a `bool` using `FromStr`.
        ArgDef::positional("third-is-better", &mut third)
            .help("Whether the third argument is better than the rest."),
        
        // Declare that 0 or more trailing arguments should be parsed as i32
        // and added to the vector 'numbers'.
        ArgDef::trail("numbers", true, &mut numbers)
            .help("A bunch of numbers used for nefarious machinations."),
        
        // Declare an '--include' option that takes a String value and adds
        // it to the HashSet 'includes'.
        // The option can be given multiple times to add more values. 
        ArgDef::collector("include", &mut includes)
            // Adds '-i' as an alias for this option.
            .short("i")
            // Sets the parameter name for this option to 'file' in the generated
            // help messages. 
            // ('--include file' instead of '--include INCLUDE')
            .param("file")
            .help("Which files to include in the cake."),
        
        // Declare a '--cool THING' setting, that sets an optional setting
        // to a value. 
        // Settings can only be set ONCE.
        ArgDef::setting("cool", &mut cool_thing).param("thing")
            .help("Something that you think is cool enough to pass."),
        
        // Declare a '--star' counter, that counts the number of times '--star'
        // or '-s' was passed, and increments 'stars' each time.
        // Counter arguments can be passed multiple times.
        ArgDef::counter("star", &mut stars).short("s")
            .help("How many stars does this library deserve?"),
        
        // Declare a '--verbose' flag, that sets a value to 'true' if passed.
        ArgDef::flag("verbose", &mut verbose).short("v")
            .help("Print as much information as possible."),
        
        
        // Declare a '--help' 'interrupt' argument, that uses 'description' as a 
        // description of the program.
        // 
        // When '--help' is passed, the parse is interrupted and 
        // Err(Interrupted("help")) will be returned.
        // 
        // As this 'interrupts' the parsing, the required values,
        // 'one', 'two' and 'third-is-better', will not have been set.
        help_arg(description).short("h"),
        
        // Declare a default '--version' argument that prints the SemVer version
        // from 'Cargo.toml' and interrupts.
        version_arg(),
    ]) {
        // If the parse has succeeded, all required (positional) values will
        // have been assigned to their target variables.
        // 
        // If any subcommands are defined, they may return an error code, so
        // that process::exit can be called after everything is cleaned up.
        Ok(_optional_error_code) => {},
        
        // If the parse receives an 'interrupt' flag, the required values will
        // not have been set, and the program should exit.
        Err(ParseError::Interrupted(_)) => {
            return None;
        },
        
        // The regular 'parse' function will print out information about errors
        // and print a usage string if an error is encountered, so in this case
        // a general error code should just be returned.
        Err(_) => {
            return Some(1);
        }
    };
    
    // If the parse did not return an error, all required values will have been
    // set to a value from an argument by this point.
    
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
    
    None
}
