# Argonaut
An argument parser for Rust, that grants as much control over the parser as possible.

## Argument conversion
This means that the arguments are *not* converted to other types (except for switches that are boolean by default).

## Help messages
It also means that help messages are not handled either. Just write it yourself, and make it **NICE!**

## Error handling
The actual argument parsing returns errors that should be pretty simple to convey to users, but these are not handled by the parser either.

Adding arguments to the parser and accessing arguments on the parsed arguments will only return an error string, as they may only have *logical* errors, such as adding arguments that would overwrite each other, or trying to access a parsed argument using an invalid identifier.

# Example
This can be found in *examples/main.rs* as well, and be run with 
```cargo run --example main -- foo bar -x baz --verbose -- arg --help```.
You can also try running it without the arguments, but these arguments will make the parse **succeed**.

```rust
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