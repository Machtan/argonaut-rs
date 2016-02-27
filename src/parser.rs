use std::collections::{HashMap, HashSet};
use common::{FlagName, OptName};
use arg::{self, Arg};
use std::env;
use std::borrow::{Cow, Borrow};


/// The possible types of an optional argument.
#[derive(Debug, Clone)]
enum OptType {
    Single,
    ZeroPlus,
    OnePlus,
}

/// Returns the flag names that might denote this option.
fn optional_flag_names<'a>(name: OptName<'a>) -> Vec<FlagName<'a>> {
    use common::FlagName::*;
    match name {
        OptName::Normal(long) => vec![Long(long)],
        OptName::NormalAndShort(long, ch) => vec![Short(ch), Long(long)],
    }
}

/// The possible types of a required argument that isn't positional.
#[derive(Debug)]
enum ReqType {
    ZeroPlus,
    OnePlus,
}



/// Creates an argument name (fat pointer) to the given argument if it is
/// valid as such.
fn argument_type<'a>(arg: &'a str) -> GivenArgument<'a> {
    use self::GivenArgument::*;
    use common::FlagName::*;
    if arg.starts_with("--") {
        Flag(Long(&arg[2..]))

    } else if arg.starts_with("-") {
        if arg.len() == 2 {
            Flag(Short(arg.chars().nth(1).unwrap()))
        } else {
            ShortFlags(arg.chars().skip(1).map(|ch| Short(ch)).collect())
        }

    } else {
        Value(arg)
    }
}

/// An error found when attempting to parse a set of arguments.
#[derive(Debug)]
pub enum ParseError<'a> {
    /// This optional argument is not recognized by the parser.
    UnknownOptionalArgument { arg: &'a str },
    /// The given short flag takes input and therefore cannot be grouped when
    /// used (if '-x' takes the argument 'FOO', you cannot call '-vasx').
    GroupedNonSwitch { arg: &'a str, invalid: String },
    /// This argument is missing a parameter.
    MissingParameter { arg: &'a str },
    /// This positional argument wasn't given.
    MissingPositionalArgument { arg: &'a str },
    /// This optional argument was given twice.
    DuplicatePositionalArgument { arg: &'a str },
    /// The required trail argument is missing.
    MissingTrail { arg: &'a str },
    /// The given positional argument was not expected by the parser.
    UnexpectedArgument { arg: &'a str },
}

/// An argument given by the user.
#[derive(Debug)]
enum GivenArgument<'a> {
    Value(&'a str),
    Flag(FlagName<'a>),
    ShortFlags(Vec<FlagName<'a>>),
}

/// An argument parser.
#[derive(Debug)]
pub struct Parser<'a> {
    positional: Vec<&'a str>,
    trail: Option<(&'a str, ReqType)>,
    options: HashMap<OptName<'a>, OptType>,
    switches: HashSet<OptName<'a>>,
    interrupts: HashSet<OptName<'a>>,
    used_flags: HashSet<FlagName<'a>>,
    aliases: HashMap<FlagName<'a>, OptName<'a>>,
    passalongs: HashSet<OptName<'a>>,
    definitions: Vec<Arg<'a>>,
}

/// One or more arguments structured by the parser.
#[derive(Debug)]
pub enum StructuredArgument<'a> {
    /// A positional argument.
    Positional { name: &'a str, value: &'a str },
    /// The trail of arguments left after all the positional arguments have been
    /// found.
    Trail { values: Vec<&'a str> },
    /// An optional argument taking a single value.
    Single { name: &'a str, parameter: &'a str },
    /// An optional argument taking multiple values.
    Multiple { name: &'a str, parameters: &'a [Cow<'a, str>] },
    /// An optional argument that is present.
    Switch { name: &'a str },
    /// An optional argument which interrupt the parse when encountered.
    Interrupt { name: &'a str },
    /// An optional argument which collects all following arguments without
    /// parsing them (for parsing arguments along to a subcommand or alike).
    PassAlong { name: &'a str, args: &'a [Cow<'a, str>] },
}

/// An iterator over structured arguments during a parse.
#[derive(Debug)]
pub struct Parse<'a> {
    index: usize,
    position: usize,
    parser: &'a Parser<'a>,
    args: Vec<Cow<'a, str>>,
    found_flags: HashSet<OptName<'a>>,
    leftover_short_flags: Vec<FlagName<'a>>,
    finished: bool,
    trail: Vec<&'a str>,
    passalong: Option<(&'a str, usize)>,
}

impl<'a> Parse<'a> {
    /// Returns the remaining unparsed arguments for this parse run.
    pub fn remaining_args(&'a self) -> &'a [Cow<'a, str>] {
        &self.args[self.index..]
    }

    // Parses the given flag
    fn parse_flag(&mut self, flag: FlagName<'a>, arg: &'a str)
            -> Result<StructuredArgument<'a>, ParseError<'a>> {
        use self::ParseError::*;
        use self::StructuredArgument::*;

        let opt_name = match self.parser.aliases.get(&flag) {
            Some(name) => *name,
            None => {
                self.finished = true;
                return Err(UnknownOptionalArgument { arg: arg });
            }
        };

        if self.found_flags.contains(&opt_name) {
            return Err(DuplicatePositionalArgument {
                arg: arg }
            );
        }

        if self.parser.switches.contains(&opt_name) {
            self.found_flags.insert(opt_name.clone());
            return Ok(Switch { name: opt_name.name() });

        } else if self.parser.interrupts.contains(&opt_name) {
            self.finished = true;
            return Ok(Interrupt { name: opt_name.name() });

        } else if self.parser.passalongs.contains(&opt_name) {
            if let Some(res) = self.check_trail() {
                self.passalong = Some((opt_name.name(), self.index));
                return res;
            } else {
                return Ok(PassAlong {
                    name: opt_name.name(), args: &self.args[self.index..]
                });
            }
        }
        // The argument must be an optional one
        self.found_flags.insert(opt_name);
        let opt_type = self.parser.options.get(&opt_name)
            .expect("Broken invariant: a flag was in aliases, but was not found\
            in the arg type structures");
        self.find_parameters(arg, opt_type, opt_name)
    }

    fn validate_grouped_short(&mut self, flag: FlagName<'a>, arg: &'a str)
            -> Result<(), ParseError<'a>> {
        use self::ParseError::*;
        let opt_name = match self.parser.aliases.get(&flag) {
            Some(name) => name,
            None => {
                self.finished = true;
                return Err(UnknownOptionalArgument { arg: arg });
            }
        };
        if ! self.parser.switches.contains(&opt_name) {
            return Err(GroupedNonSwitch {
                arg: arg, invalid: flag.to_string()
            });
        }
        Ok(())
    }

    fn check_trail(&mut self)
            -> Option<Result<StructuredArgument<'a>, ParseError<'a>>> {
        use self::StructuredArgument::*;
        use self::ParseError::*;
        // A positional argument wasn't given
        if self.position < self.parser.positional.len() {
            let arg = self.parser.positional[self.position];
            return Some(Err(MissingPositionalArgument { arg: arg }));
        }
        match self.parser.trail {
            // Validate that at least one trail argument is present
            Some((arg, ReqType::OnePlus)) => {
                if self.trail.len() < 1 {
                    return Some(Err(MissingTrail { arg: arg }));
                }
            },
            Some((_, ReqType::ZeroPlus)) => {},
            // No trail expected and none found: just return
            None => {
                return None;
            }
        }
        // Return the trail
        return Some(Ok(Trail { values: self.trail.clone() }));
    }

    /// Attempts to find enough parameters for the given option type.
    fn find_parameters(&mut self, arg: &'a str,
            opt_type: &OptType, opt_name: OptName<'a>)
            -> Result<StructuredArgument<'a>, ParseError<'a>> {
        use self::ParseError::*;
        use self::StructuredArgument::*;
        use self::GivenArgument::Value;
        let args = &self.args[self.index..];
        //println!("Finding parameters of {} ({:?}) in {:?}", name, opt_type, args);
        match *opt_type {
            OptType::Single => {
                self.index += 1;
                if args.len() < 1 {
                    return Err(MissingParameter { arg: arg });
                }
                if let Value(value) = argument_type(args[0].borrow()) {
                    return Ok(Single {
                        name: opt_name.name(),
                        parameter: value
                    });
                } else {
                    return Err(MissingParameter { arg: arg });
                }
            },
            OptType::ZeroPlus => {
                let count = args.iter().take_while(|arg| {
                    if let Value(_) = argument_type(arg) {
                        true
                    } else {
                        false
                    }
                }).count();
                let params = &self.args[self.index .. self.index+count];
                self.index += count;
                return Ok(Multiple {
                    name: opt_name.name(),
                    parameters: params,
                });
            },
            OptType::OnePlus => {
                if args.len() < 1 {
                    return Err(MissingParameter { arg: arg });
                }
                if let Value(_) = argument_type(args[0].borrow()) {
                } else {
                    return Err(MissingParameter { arg: arg });
                }
                let count = 1 + args.iter().skip(1).take_while(|arg| {
                    if let Value(_) = argument_type(arg) {
                        true
                    } else {
                        false
                    }
                }).count();
                let params = &self.args[self.index .. self.index+count];
                self.index += count;
                return Ok(Multiple {
                    name: opt_name.name(),
                    parameters: params,
                });
            },
        }
    }
}

impl<'a> Iterator for Parse<'a> {
    type Item = Result<StructuredArgument<'a>, ParseError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        use self::GivenArgument::*;
        use self::StructuredArgument::*;
        use self::ParseError::*;

        // Stop if the parse is finished
        if self.finished {
            return None;
        }

        // Check for leftover short flag from grouped short switches eg. '-abc'
        if self.leftover_short_flags.len() > 0 {
            let flag = self.leftover_short_flags.remove(0);
            let arg = self.args[self.index - 1];
            match self.validate_grouped_short(flag, arg.borrow()) {
                Err(err) => return Some(Err(err)),
                _ => {}
            }
            return Some(self.parse_flag(flag, arg.borrow()))
        }

        // Check for a leftover passalong argument
        if let Some((name, index)) = self.passalong {
            self.finished = true;
            return Some(Ok(PassAlong {
                name: name,
                args: &self.args[index..]
            }));
        }

        while self.index < self.args.len() {
            let arg = self.args[self.index];
            self.index += 1;
            match argument_type(arg.borrow()) {
                Value(value) => {
                    // Trail?
                    if self.position >= self.parser.positional.len() {
                        if let Some(_) = self.parser.trail {
                            self.trail.push(value);
                        } else {
                            self.finished = true;
                            return Some(Err(UnexpectedArgument { arg: value }));
                        }
                    // Positional
                    } else {
                        let position = self.parser.positional[self.position];
                        self.position += 1;
                        return Some(Ok(Positional {
                            name: position, value: value
                        }));
                    }
                },
                Flag(flag) => {
                    return Some(self.parse_flag(flag, arg.borrow()));
                },
                ShortFlags(flags) => {
                    self.leftover_short_flags = flags;
                    let flag = self.leftover_short_flags.remove(0);
                    match self.validate_grouped_short(flag, arg.borrow()) {
                        Err(err) => return Some(Err(err)),
                        _ => {}
                    }
                    return Some(self.parse_flag(flag, arg.borrow()));
                },
            }
        }

        if ! self.finished {
            self.finished = true;
            self.check_trail()
        } else {
            self.finished = true;
            None
        }
    }
}

impl<'a> Parser<'a> {
    /// Creates a new parser.
    pub fn new() -> Self {
        Parser {
            positional: Vec::new(),
            trail: None,
            options: HashMap::new(),
            switches: HashSet::new(),
            interrupts: HashSet::new(),
            used_flags: HashSet::new(),
            aliases: HashMap::new(),
            passalongs: HashSet::new(),
            definitions: Vec::new(),
        }
    }

    /// Adds a list of argument definitions to the parser.
    /// Errors if an optional argument with the same name has already been
    /// added, or if a trail is added twice.
    pub fn define(&mut self, args: &[Arg<'a>]) -> Result<(), String> {
        for arg in args {
            try!(self.define_single(*arg));
        }
        Ok(())
    }

    /// Adds an argument definition to the parser.
    /// Errors if an optional argument with the same name has already been
    /// added, or if a trail is added twice.
    pub fn define_single(&mut self, arg: Arg<'a>) -> Result<(), String> {
        use arg::ArgType::*;

        if let Some(optname) = arg.option_name() {
            let names = optional_flag_names(optname);

            for name in names.iter() {
                if self.used_flags.contains(name) {
                    return Err(format!(
                        "The flag '{}' is already defined", name
                    ));
                }
            }

            for name in names.iter() {
                self.used_flags.insert(name.clone());
                self.aliases.insert(name.clone(), optname.clone());
            }
        }

        match arg::internal_get_raw(arg) {
            Single(name) => {
                if self.positional.contains(&name) {
                    return Err(format!(
                        "A positional argument with the name '{}' has already \
                        been added", name
                    ));
                } else {
                    self.positional.push(name);
                }
            },
            ZeroPlus(name) => {
                match self.trail {
                    Some(_) => {
                        return Err(format!(
                            "A trailing argument has already been set"
                        ));
                    },
                    None => {
                        self.trail = Some((name, ReqType::ZeroPlus));
                    }
                }
            },
            OnePlus(name) => {
                match self.trail {
                    Some(_) => {
                        return Err(format!(
                            "A trailing argument has already been set"
                        ));
                    },
                    None => {
                        self.trail = Some((name, ReqType::OnePlus));
                    }
                }
            },
            Switch(optname) => {
                self.switches.insert(optname);
            },
            Interrupt(optname) => {
                self.interrupts.insert(optname);
            },
            PassAlong(optname) => {
                self.passalongs.insert(optname);
            },
            OptSingle(optname) => {
                self.options.insert(optname, OptType::Single);
            },
            OptZeroPlus(optname) => {
                self.options.insert(optname, OptType::ZeroPlus);
            },
            OptOnePlus(optname) => {
                self.options.insert(optname, OptType::OnePlus);
            },
        }
        self.definitions.push(arg);
        Ok(())
    }

    /// Starts parsing the given arguments with this parser.
    pub fn parse<T>(&'a self, args: &[T]) -> Parse<'a> where T: Into<Cow<'a, str>> {
        let mut arg_vec = Vec::new();
        for arg in args {
            arg_vec.push((*arg).into());
        }
        Parse {
            index: 0,
            position: 0,
            parser: self,
            args: arg_vec,
            found_flags: HashSet::new(),
            leftover_short_flags: Vec::new(),
            finished: false,
            trail: Vec::new(),
            passalong: None,
        }
    }

    /*
    /// Starts parsing the arguments in env::args() (from 1 and onwards) with this parser.
    pub fn parse_env(&'a self) -> Parse<'a> {
        parse.env_args = env::args().skip(1).collect();
        parse.env_ref.clear();
        for arg in parse.env_args.iter() {
            parse.env_ref.push(arg);
        }
        parse.args = &parse.env_ref;
        parse
    }*/
}

pub fn internal_get_definitions<'a, 'b>(parser: &'b Parser<'a>)
        -> &'b Vec<Arg<'a>> {
    &parser.definitions
}
