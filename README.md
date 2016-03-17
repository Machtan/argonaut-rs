# Argonaut
A minimal argument parser for Rust.

## Argument conversion
This means that the arguments are *not* converted to other types.

## Help messages
It also means that help messages are not handled either. There used to be an optional generator of help messages, but it is broken at the moment.

## Error handling
Errors are returned during the parse, so that the user may handle them explicitly.
There is also small validation step when adding definitions for expected arguments, but just to ensure that they are logically sound.

# Example
This can be found in *examples/main.rs* as well, and be run with 
```cargo run --example main -- foo bar -x baz --verbose -- arg --help```.
You can also try running it without the arguments, but these arguments will make the parse **succeed**.

```rust
extern crate argonaut;

use std::env;
use argonaut::{ArgDef, Parse};

const USAGE: &'static str = "Usage: cargo run --example main -- \
    [--help | OPTIONS ] foo [foobar, ...]";

const HELP: &'static str = "\
Required arguments:
    foo                     A single positional argument.
    foobar [foobar ...]     One or more trailing arguments.

Optional arguments:
    --help | -h         Show this message.
    --version           Show the version of this library.
    --verbose | -v      Set the 'verbose' flag to true.
    --exclude | -x ARG  Do something with a single parameter.  
    --                  Collect all following arguments verbatim.\
";

fn main() {
    use argonaut::Arg::*;

    // Create the arguments
    let a_foo       = ArgDef::positional("foo");
    let a_foobar    = ArgDef::required_trail();
    let a_version   = ArgDef::named("version").switch();
    let a_passed    = ArgDef::named("").switch();
    let a_help      = ArgDef::named_and_short("help", 'h').switch();
    let a_verbose   = ArgDef::named_and_short("verbose", 'v').switch();
    let a_exclude   = ArgDef::named_and_short("exclude", 'x').option();
    
    // IMPORTANT: the arguments to the parser must outlive the variables
    // that are set, as the arguments are referenced rather than copied/cloned.
    let args: Vec<_> = env::args().skip(1).collect();
    
    // Prepare the options
    let mut foo = "";
    let mut foobar = Vec::new();
    let mut verbose = false;
    let mut exclude = None;
    let mut passed = None;
    
    let expected = &[a_foo, a_foobar, a_help, a_version, a_verbose, a_exclude,
         a_passed];
    
    // Avoid consuming the parse iterator, in order to get the remaining
    // arguments when encountering the '--' flag.
    // When the parse is iterated instead, all arguments are consumed, so I
    // prefer the ```while let Some(item) = parse.next()``` form.
    
    let mut parse = Parse::new(expected, &args).expect("Invalid definitions");
    while let Some(item) = parse.next() {
        match item {
            Err(err) => {
                println!("Parse error: {:?}", err);
                println!("{}", USAGE);
                return;
            },
            // When the positional argument named ```foo``` is found.
            Ok(Positional("foo", value)) => {
                foo = value;
            },
            // When a trailing argument is found (and a trail is expected).
            Ok(TrailPart(value)) => {
                foobar.push(value);
            },
            // When the switch ```--help``` is found.
            // This is also given for the short version ```-h```.
            Ok(Switch("help")) => {
                return println!("{}\n\n{}", USAGE, HELP);
            },
            Ok(Switch("version")) => {
                return println!("{}", env!("CARGO_PKG_VERSION"));
            },
            Ok(Switch("verbose")) => {
                verbose = true;
            },
            // When the option (switch with an argument) ```exclude``` is found.
            Ok(Option("exclude", value)) => {
                exclude = Some(value);
            },
            Ok(Switch("")) => {
                // Take a reference to a slice of the remaining unparsed arguments.
                passed = Some(parse.remainder());
                break;
            },
            _ => unreachable!(),
        }
    }
    
    // Use the variables holding the parsed values for something
    println!("Parsed succesfully!");
    println!("Foo:          {}", foo);
    println!("Foobar:       {:?}", foobar);
    println!("Verbose:      {}", verbose);
    println!("Exclude:      {:?}", exclude);
    println!("Passed args:  {:?}", passed);
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.