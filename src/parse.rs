use std::borrow::{Cow, Borrow};
use std::collections::{HashMap, HashSet};
use std::env;

/// A parsed argument.
// Lifetime 'a is the "definitions" and 'b is the "arguments".
#[derive(Debug)]
pub enum Arg<'a, 'b> {
    Positional { name: &'a str, value: Cow<'b, str> },
    Trail { value: Cow<'b, str> },
    Switch { name: &'a str },
    Option { name: &'a str, value: Cow<'b, str> },
}

/// The name of an optional argument.
#[derive(Debug, Clone)]
pub enum OptName<'a> {
    Long(Cow<'a, str>),
    LongAndShort(Cow<'a, str>, char),
}

impl<'a> OptName<'a> {
    /// The 'long' component of the name.
    pub fn borrow_long(&'a self) -> &'a str {
        match *self {
            OptName::Long(ref name) | OptName::LongAndShort(ref name, _) => {
                name.borrow()
            },
        }
    }
}

/// The different argument structures to expect.
#[derive(Debug, Clone)]
pub enum DefType<'a> {
    Positional { name: Cow<'a, str> },
    Trail(TrailType),
    Switch { name: OptName<'a> },
    Option { name: OptName<'a> },
}

#[must_use = "The argument definition is only partially constructed"]
pub struct PartialArgDef<'a> {
    pub name: OptName<'a>
}

impl<'a> PartialArgDef<'a> {
    pub fn switch(self) -> ArgDef<'a> {
        ArgDef::new(DefType::Switch { name: self.name })
    }
    
    pub fn option(self) -> ArgDef<'a> {
        ArgDef::new(DefType::Option { name: self.name })
    }
}

/// The definition of one or more arguments to expect when parsing.
#[derive(Debug, Clone)]
pub struct ArgDef<'a> {
    pub deftype: DefType<'a>,
    pub help: Option<Cow<'a, str>>,
    pub parameter: Option<Cow<'a, str>>,
}

impl<'a> ArgDef<'a> {
    fn new(deftype: DefType<'a>) -> ArgDef<'a> {
        ArgDef {
            deftype: deftype,
            help: None,
            parameter: None,
        }
    } 
    
    pub fn positional<T>(name: T) -> ArgDef<'a> where T: Into<Cow<'a, str>> {
        ArgDef::new(DefType::Positional { name: name.into() })
    }
    
    pub fn required_trail() -> ArgDef<'a> {
        ArgDef::new(DefType::Trail(TrailType::OnePlus))
    }
    
    pub fn optional_trail() -> ArgDef<'a> {
        ArgDef::new(DefType::Trail(TrailType::ZeroPlus))
    }
    
    pub fn named<T>(name: T) 
            -> PartialArgDef<'a> where T: Into<Cow<'a, str>> {
        PartialArgDef { name: OptName::Long(name.into()) }
    }
    
    pub fn named_and_short<T>(name: T, short: char)
            -> PartialArgDef<'a> where T: Into<Cow<'a, str>> {
        PartialArgDef { name: OptName::LongAndShort(name.into(), short) }
    }
    
    pub fn set_help<T>(&mut self, help: T) 
            -> &mut Self 
            where T: Into<Cow<'a, str>> {
        self.help = Some(help.into());
        self
    }
    
    pub fn set_parameter<T>(&mut self, param: T) 
            -> &mut Self 
            where T: Into<Cow<'a, str>> {
        self.parameter = Some(param.into());
        self
    }
}

/// The types of argument trails to expect.
#[derive(Debug, Clone)]
pub enum TrailType {
    OnePlus,
    ZeroPlus,
}

/// An error found when defining the expected argument structure.
#[derive(Debug)]
pub enum DefinitionError<'a> {
    SameShortName(&'a str, &'a str),
    DefinedTwice(&'a str),
    TwoTrailsDefined,
}

/// The type of an optional argument.
#[derive(Debug, Clone)]
enum OptType {
    Switch,
    Option,
}

/// An error found when parsing.
#[derive(Debug)]
pub enum ParseError<'a, 'b> {
    MissingPositional(&'a str),
    MissingTrail,
    UnexpectedPositional(Cow<'b, str>),
    UnknownShortArgument(char, Cow<'b, str>),
    UnknownLongArgument(Cow<'b, str>),
    MissingParameter(Cow<'b, str>),
    GroupedNonSwitch(char, Cow<'b, str>),
}

/// A parse of a set of string arguments.
// Lifetime 'a is the "definitions" and 'b is the "arguments".
#[derive(Debug, Clone)]
pub struct Parse<'a, 'b> {
    positional: Vec<&'a str>,
    next_position: usize,
    trail: Option<TrailType>,
    options: HashMap<&'a str, OptType>, 
    aliases: HashMap<char, &'a str>,
    reversed_args: Vec<Cow<'b, str>>,
    remaining_grouped_shorts: Option<(Cow<'b, str>, Vec<char>)>,
    finished: bool,
    trail_args_found: usize,
}



impl<'a, 'b> Parse<'a, 'b> {
    
    fn add_name(name: &'a OptName<'a>, aliases: &mut HashMap<char, &'a str>, 
            used_names: &mut HashSet<Cow<'a, str>>) 
            -> Result<(), DefinitionError<'a>> {
        use self::DefinitionError::*;
        match *name {
            OptName::Long(ref long) => {
                if used_names.contains(long) {
                    return Err(DefinedTwice(long.borrow()));
                } else {
                    used_names.insert(long.clone());
                }
            },
            OptName::LongAndShort(ref long, short) => {
                if used_names.contains(long) {
                    return Err(DefinedTwice(long.borrow()));
                } else {
                    used_names.insert(long.clone());
                }
                if let Some(other_long) = aliases.get(&short) {
                    return Err(SameShortName(long.borrow(), other_long.clone()));
                }
                aliases.insert(short, long.borrow());
            }
        }
        Ok(())
    }
    
    /// Starts a new parse checking for the expected argument structure among
    /// the given list of arguments.
    pub fn new<T>(expected: &'a [ArgDef<'a>], args: &[T])
            -> Result<Parse<'a, 'b>, DefinitionError<'a>> 
            where T: Clone + Into<Cow<'b, str>> {
        use self::DefinitionError::*;
        use self::DefType::*;
        
        let mut options = HashMap::new();
        let mut positional = Vec::new();
        let mut trail = None;
        
        let mut aliases = HashMap::new();
        let mut used_names = HashSet::new();
        for def in expected {
            match def.deftype {
                Positional { ref name } => {
                    positional.push(name.borrow());
                },
                Trail(ref trail_type) => {
                    if trail.is_some() {
                        return Err(TwoTrailsDefined);
                    } else {
                        trail = Some(trail_type.clone());
                    }
                },
                Switch { ref name } => {
                    try!(Parse::add_name(name, &mut aliases, &mut used_names));
                    options.insert(name.borrow_long().clone(), OptType::Switch);
                },
                Option { ref name } => {
                    try!(Parse::add_name(name, &mut aliases, &mut used_names));
                    options.insert(name.borrow_long().clone(), OptType::Option);
                },
            }
        }
        
        Ok(Parse {
            options: options,
            positional: positional,
            next_position: 0,
            trail: trail,
            aliases: aliases,
            reversed_args: args.iter().rev()
                .map(|a: &T| (*a).clone().into()).collect(),
            remaining_grouped_shorts: None,
            finished: false,
            trail_args_found: 0,
        })
    }
    
    /// Creates a parse of the arguments given to the program at launch.
    pub fn new_from_env(expected: &'a [ArgDef<'a>])
            -> Result<Parse<'a, 'b>, DefinitionError<'a>> {
        Parse::new(expected, &env::args().skip(1).collect::<Vec<String>>())
    }
    
    fn read_option(&mut self, name: &'a str, opt_type: &OptType, arg: Cow<'b, str>)
            -> Result<Arg<'a, 'b>, ParseError<'a, 'b>> {
        use self::OptType::*;
        use self::ParseError::*;
        match *opt_type {
            Switch => {
                Ok(Arg::Switch { name: name })
            },
            Option => {
                if self.reversed_args.is_empty() {
                    self.finished = true;
                    Err(MissingParameter(arg))
                } else {
                    if let Some(param) = self.reversed_args.pop() {
                        if param.starts_with("-") {
                            self.finished = true;
                            Err(MissingParameter(arg))
                        } else {
                            Ok(Arg::Option { name: name, value: param })
                        }
                    } else {
                        self.finished = true;
                        Err(MissingParameter(arg))
                    }
                }
            },
        }
    }
    
    /// Ends this parse and returns the remaining arguments.
    pub fn finish(mut self) -> Vec<Cow<'b, str>> {
        self.reversed_args.reverse();
        self.reversed_args
    }
}

impl<'a, 'b> Iterator for Parse<'a, 'b> {
    type Item = Result<Arg<'a, 'b>, ParseError<'a, 'b>>;

    /// Attempts to read the next argument for this parse
    fn next(&mut self) -> Option<Self::Item> {
        use self::Arg::*;
        use self::ParseError::*;
        
        if self.finished {
            return None;
        }
        
        let is_empty = if let Some((_, ref shorts)) = self.remaining_grouped_shorts {
            shorts.is_empty()
        } else {
            false
        };
        if is_empty {
            self.remaining_grouped_shorts = None;
        }
        
        if let Some((ref arg, ref mut shorts)) = self.remaining_grouped_shorts {
            let short = shorts.pop().expect("shorts not cleared");
            
            let name = if let Some(name) = self.aliases.get(&short) {
                name.clone()
            } else {
                self.finished = true;
                return Some(Err(UnknownShortArgument(short, arg.clone())));
            };
            if let OptType::Option = *self.options.get(name).expect("Invariant broken") {
                self.finished = true;
                return Some(Err(GroupedNonSwitch(short, arg.clone())));
            } else {
                return Some(Ok(Switch { name: name }));
            }
        }
        
        let arg = if let Some(arg) = self.reversed_args.pop() {
            arg
        // No more arguments
        } else {
            self.finished = true;
            // Missing positional
            if self.next_position < self.positional.len() {
                let pos = self.positional[self.next_position];
                return Some(Err(MissingPositional(pos)));
            // Missing trail
            } else if let Some(TrailType::OnePlus) = self.trail {
                if self.trail_args_found == 0 {
                    return Some(Err(MissingTrail));
                }
            }
            
            return None;
        };
        
        // Long argument
        if arg.starts_with("--") {
            let opt_type = if let Some(opt_type) = self.options.get(&arg[2..]) {
                opt_type.clone()
            } else {
                self.finished = true;
                return Some(Err(UnknownLongArgument(arg.clone())));
            };
            // Exchange the "argument" reference with the "definition" one
            let key = self.options.keys().find(|n| *n == &&arg[2..]).unwrap().clone();
            return Some(self.read_option(key, &opt_type, arg));
       
        // Short argument
        } else if arg.starts_with("-") {
            let mut shorts: Vec<_> = arg.chars().skip(1).collect();
            if shorts.len() > 1 { // grouped short args
                shorts.reverse();
                self.remaining_grouped_shorts = Some((arg, shorts));
            } else {
                let short = shorts[0];
                let name = if let Some(name) = self.aliases.get(&short) {
                    name.clone()
                } else {
                    self.finished = true;
                    return Some(Err(UnknownShortArgument(short, arg.clone())));
                };
                let opt_type = self.options.get(name).expect("Invariant broken").clone();
                return Some(self.read_option(name, &opt_type, arg));
            }
        
        // Positional argument
        } else {
            // Positions left to be filled
            if self.next_position < self.positional.len() {
                let position = self.positional[self.next_position];
                self.next_position += 1;
                return Some(Ok(Positional { name: position, value: arg }));
            
            // Trail
            } else if let Some(_) = self.trail {
                self.trail_args_found += 1;
                return Some(Ok(Trail { value: arg }));
            
            // No trail
            } else {
                self.finished = true;
                return Some(Err(UnexpectedPositional(arg)));
            }
        }
        
        None
    }
}