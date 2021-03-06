/*!
A simple argument parsing library.

# Lifetimes
`'def`: `argument definition`

The lifetime of argument definition passed to `parse`


`'tar`: `target`

The lifetime of target pointers used when defining arguments.
*/

#![feature(conservative_impl_trait)]
#![feature(unicode)]

extern crate std_unicode;

mod argdef;
mod help;
mod parse;

pub use argdef::{ArgDef, SingleTarget, CollectionTarget, OptionTarget};
pub use parse::{parse, parse_plain, ParseError};

/*
DESIGN: Do I wait with assigning values until all arguments have been 'satisfied'?
Or do I just start parsing/assigning as soon as possible so that bad arguments
are caught faster?
For now it'll be 2, since that seems simpler

# option 1
read through the arguments and assign each to a matching option
if an interrupt is encountered: 
    run the callback and return the interrupt
validate each argument (add 'validate' to the interface)
go through and parse every value into its target
return success
*/

/*
Tasks

Optional
- Implement a validate->assign->modify procedure in parse
- Make a passthrough argument (cargo run -- --help)
- Add examples to README and library top-level documentation
- Add tests

Done
- Usage generator (printer)
- Help generator (printer)
- Simple subcommand abstraction
- Validate 'short' identifiers
- Change default parse function to write usage (no parse_subcommand)
- Implement multi-target for all std::collections
- Let parse results pass either Option<i32> or Option<T> to facilitate 'outer' abort.
- Make a 'collect' argument (gcc -i foo.h -i bar.h)

Abandoned
- Make a default handler function for parse results.

*/


