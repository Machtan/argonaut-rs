extern crate argonaut;

use std::env;
use argonaut::{ArgDef, Parse};

fn main() {
    use argonaut::Arg::*;
    println!("Argonaut!");

    // Create the arguments
    let a_foo = ArgDef::positional("foo");
    let a_foobar = ArgDef::required_trail();
    let a_help = ArgDef::named_and_short("help", 'h').switch();
    let a_version = ArgDef::named("version").switch();
    let a_verbose = ArgDef::named_and_short("verbose", 'v').switch();
    let a_exclude = ArgDef::named_and_short("exclude", 'x').option();
    let a_passed = ArgDef::named("").switch();
    
    // IMPORTANT: the arguments to the parser must outlive the variables
    // that are set, as the arguments are referenced rather than copied/cloned.
    let args: Vec<_> = env::args().skip(1).collect();
    
    // The default return values are cows.
    let mut foo = "";
    let mut foobar = Vec::new();
    let mut verbose = false;
    let mut exclude = None;
    let mut passed = None;
    
    let usage = "Usage: cargo run --example main -- [--help | OPTIONS ] foo [foobar, ...]";
    let expected = &[a_foo, a_foobar, a_help, a_version, a_verbose, a_exclude,
         a_passed];
    
    // Avoid consuming the parse iterator, in order to get the remaining
    // arguments when encountering the '--' flag
    /*let args = vec![
        String::from("foo"),
        String::from("bar"),
        String::from("-x"),
        String::from("baz"),
        String::from("--verbose"),
        String::from("--"),
        String::from("arg"),
        String::from("--help]"),
    ];*/
    let mut parse = Parse::new(expected, &args).expect("Invalid definitions");
    while let Some(item) = parse.next() {
        match item {
            Err(err) => {
                println!("Parse error: {:?}", err);
                println!("{}", usage);
                return;
            },
            Ok(Positional { name: "foo", value }) => {
                foo = value;
            },
            Ok(Trail { value }) => {
                foobar.push(value);
            },
            Ok(Switch { name: "help" }) => {
                return println!("{}\n\n{}", usage, "No help atm.");
            },
            Ok(Switch { name: "version" }) => {
                return println!("{}", env!("CARGO_PKG_VERSION"));
            },
            Ok(Switch { name: "verbose" }) => {
                verbose = true;
            },
            Ok(Option { name: "exclude", value }) => {
                exclude = Some(value);
            },
            Ok(Switch { name: "" }) => {
                passed = Some(parse.remainder());
                break;
            },
            _ => unreachable!(),
        }
    }
    // Use the parsed values
    println!("Parsed succesfully!");
    println!("Foo:          {}", foo);
    println!("Foobar:       {:?}", foobar);
    println!("Verbose:      {}", verbose);
    println!("Exclude:      {:?}", exclude);
    println!("Passed args:  {:?}", passed);
}
