extern crate argonaut;

use std::borrow::Cow;
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

    // The default return values are cows.
    let mut foo = Cow::from("");
    let mut foobar = Vec::new();
    let mut verbose = false;
    let mut exclude = None;
    let mut passed = None;
    
    let usage = "Usage: cargo run --example main -- [--help | OPTIONS ] foo [foobar, ...]";
    let expected = &[a_foo, a_foobar, a_help, a_version, a_verbose, a_exclude,
         a_passed];
    
    // Avoid consuming the parse iterator, in order to get the remaining
    // arguments when encountering the '--' flag
    let mut parse = Parse::new_from_env(expected).expect("Invalid definitions");
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
                passed = Some(parse.finish());
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
