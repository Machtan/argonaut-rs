use std::borrow::{Borrow};
use std::collections::{HashMap, HashSet};

/// A parsed argument.
// Lifetime 'a is the "definitions" and 'b is the "arguments".
#[derive(Debug)]
pub enum Arg<'a, 'b> {
    Positional { name: &'a str, value: &'b str },
    Trail { value: &'b str },
    Switch { name: &'a str },
    Option { name: &'a str, value: &'b str },
}

/// The name of an optional argument.
#[derive(Debug, Clone)]
pub enum OptName<T: Borrow<str>> {
    Long(T),
    LongAndShort(T, char),
}

impl<T: Borrow<str>> OptName<T> {
    /// The 'long' component of the name.
    pub fn borrow_long<'a>(&'a self) -> &'a str {
        match *self {
            OptName::Long(ref name) | OptName::LongAndShort(ref name, _) => {
                name.borrow()
            },
        }
    }
}

/// The different argument structures to expect.
#[derive(Debug, Clone)]
pub enum DefType<T: Borrow<str>> {
    Positional { name: T },
    Trail(TrailType),
    Switch { name: OptName<T> },
    Option { name: OptName<T> },
}

#[must_use = "The argument definition is only partially constructed"]
pub struct PartialArgDef<T: Borrow<str>> {
    pub name: OptName<T>
}

impl<T: Borrow<str>> PartialArgDef<T> {
    pub fn switch(self) -> ArgDef<T> {
        ArgDef::new(DefType::Switch { name: self.name })
    }
    
    pub fn option(self) -> ArgDef<T> {
        ArgDef::new(DefType::Option { name: self.name })
    }
}

/// The definition of one or more arguments to expect when parsing.
#[derive(Debug, Clone)]
pub struct ArgDef<T: Borrow<str>> {
    pub deftype: DefType<T>,
    pub help: Option<T>,
    pub parameter: Option<T>,
}

impl<T: Borrow<str>> ArgDef<T> {
    fn new(deftype: DefType<T>) -> ArgDef<T> {
        ArgDef {
            deftype: deftype,
            help: None,
            parameter: None,
        }
    } 
    
    pub fn positional(name: T) -> ArgDef<T> {
        ArgDef::new(DefType::Positional { name: name })
    }
    
    pub fn required_trail() -> ArgDef<T> {
        ArgDef::new(DefType::Trail(TrailType::OnePlus))
    }
    
    pub fn optional_trail() -> ArgDef<T> {
        ArgDef::new(DefType::Trail(TrailType::ZeroPlus))
    }
    
    pub fn named(name: T) -> PartialArgDef<T> {
        PartialArgDef { name: OptName::Long(name) }
    }
    
    pub fn named_and_short(name: T, short: char) -> PartialArgDef<T> {
        PartialArgDef { name: OptName::LongAndShort(name, short) }
    }
    
    pub fn set_help(&mut self, help: T) -> &mut Self {
        self.help = Some(help);
        self
    }
    
    pub fn set_parameter(&mut self, param: T) -> &mut Self {
        self.parameter = Some(param);
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
    MissingParameter(&'a str),
    MissingTrail,
    UnexpectedPositional(&'b str),
    UnexpectedShortArgument(char, &'b str),
    UnexpectedLongArgument(&'b str),
    GroupedNonSwitch(char, &'b str),
}

/// A parse of a set of string arguments.
// Lifetime 'a is the "definitions" and 'b is the "arguments".
#[derive(Debug, Clone)]
pub struct Parse<'a, 'b, T: 'b + Borrow<str>> {
    args: &'b [T],
    index: usize,
    finished: bool,
    positional: Vec<&'a str>,
    next_position: usize,
    trail: Option<TrailType>,
    trail_args_found: usize,
    options: HashMap<&'a str, OptType>, 
    aliases: HashMap<char, &'a str>,
    remaining_grouped_shorts: Vec<(usize, char)>,
}



impl<'a, 'b, T: 'b + Borrow<str>,> Parse<'a, 'b, T> {
    
    fn add_name<D>(name: &'a OptName<D>, aliases: &mut HashMap<char, &'a str>, 
            used_names: &mut HashSet<&'a str>) 
            -> Result<(), DefinitionError<'a>> 
            where D: Borrow<str> {
        use self::DefinitionError::*;
        match *name {
            OptName::Long(ref long) => {
                let name = long.borrow();
                if used_names.contains(name) {
                    return Err(DefinedTwice(name));
                } else {
                    used_names.insert(name);
                }
            },
            OptName::LongAndShort(ref long, short) => {
                let name = long.borrow();
                if used_names.contains(name) {
                    return Err(DefinedTwice(name));
                } else {
                    used_names.insert(name);
                }
                if let Some(other_long) = aliases.get(&short) {
                    return Err(SameShortName(name, other_long.clone()));
                }
                aliases.insert(short, name);
            }
        }
        Ok(())
    }
    
    /// Starts a new parse checking for the expected argument structure among
    /// the given list of arguments.
    pub fn new<D>(expected: &'a [ArgDef<D>], args: &'b [T])
            -> Result<Parse<'a, 'b, T>, DefinitionError<'a>> 
            where D: Borrow<str> {
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
                    try!(Parse::<T>::add_name(name, &mut aliases, &mut used_names));
                    options.insert(name.borrow_long().clone(), OptType::Switch);
                },
                Option { ref name } => {
                    try!(Parse::<T>::add_name(name, &mut aliases, &mut used_names));
                    options.insert(name.borrow_long().clone(), OptType::Option);
                },
            }
        }
        
        Ok(Parse {
            args: args,
            index: 0,
            finished: false,
            positional: positional,
            next_position: 0,
            trail: trail,
            trail_args_found: 0,
            options: options,
            aliases: aliases,
            remaining_grouped_shorts: vec![], 
        })
    }
    
    fn read_option(&mut self, name: &'a str, opt_type: &OptType)
            -> Result<Arg<'a, 'b>, ParseError<'a, 'b>> {
        use self::OptType::*;
        use self::ParseError::*;
        match *opt_type {
            Switch => {
                Ok(Arg::Switch { name: name })
            },
            Option => {
                if self.args.is_empty() {
                    self.finished = true;
                    Err(MissingParameter(name.borrow()))
                } else {
                    let arg_count = self.args.len();
                    if self.index < arg_count {
                        let ref param = self.args[self.index];
                        let string = param.borrow();
                        self.index += 1;
                        if string.starts_with("-") {
                            self.finished = true;
                            Err(MissingParameter(name.clone()))
                        } else {
                            Ok(Arg::Option { name: name, value: string })
                        }
                    } else {
                        self.finished = true;
                        Err(MissingParameter(name.clone()))
                    }
                }
            },
        }
    }
    
    /// Returns the remaining unparsed arguments of this parse.
    pub fn remainder(&self) -> &'b [T] {
        &self.args[self.index..]
    }
}

impl<'a, 'b, T: 'b + Borrow<str>> Iterator for Parse<'a, 'b, T> {
    type Item = Result<Arg<'a, 'b>, ParseError<'a, 'b>>;

    /// Attempts to read the next argument for this parse
    fn next(&mut self) -> Option<Self::Item> {
        use self::Arg::*;
        use self::ParseError::*;
        
        if self.finished {
            return None;
        }
        
        // Check for extra grouped short arguments from the last parsed argument
        
        if let Some((index, short)) = self.remaining_grouped_shorts.pop() {            
            let name = if let Some(name) = self.aliases.get(&short) {
                name.clone()
            } else {
                self.finished = true;
                return Some(Err(UnexpectedShortArgument(short, self.args[index].borrow())));
            };
            if let OptType::Option = *self.options.get(name).expect("Invariant broken") {
                self.finished = true;
                return Some(Err(GroupedNonSwitch(short, self.args[index].borrow())));
            } else {
                return Some(Ok(Switch { name: name }));
            }
        }
        
        let arg_count = self.args.len();
        let arg = if self.index < arg_count {
            let ref arg = self.args[self.index];
            self.index += 1;
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
        let string = arg.borrow();
        if string.starts_with("--") {
            let opt_type = if let Some(opt_type) = self.options.get(&string[2..]) {
                opt_type.clone()
            } else {
                self.finished = true;
                return Some(Err(UnexpectedLongArgument(string.clone())));
            };
            // Exchange the "argument" reference with the "definition" one
            let key = self.options.keys().find(|n| *n == &&string[2..]).unwrap().clone();
            return Some(self.read_option(key, &opt_type));
       
        // Short argument
        } else if string.starts_with("-") {
            let mut shorts: Vec<_> = string.chars().skip(1)
                .map(|ch| (self.index - 1, ch)).collect();
            if shorts.len() > 1 { // grouped short args
                shorts.reverse();
                self.remaining_grouped_shorts = shorts;
            } else {
                let (_, short) = shorts[0];
                let name = if let Some(name) = self.aliases.get(&short) {
                    name.clone()
                } else {
                    self.finished = true;
                    return Some(Err(UnexpectedShortArgument(short, string.clone())));
                };
                let opt_type = self.options.get(name).expect("Invariant broken").clone();
                return Some(self.read_option(name, &opt_type));
            }
        
        // Positional argument
        } else {
            // Positions left to be filled
            if self.next_position < self.positional.len() {
                let position = self.positional[self.next_position];
                self.next_position += 1;
                return Some(Ok(Positional { name: position, value: string }));
            
            // Trail
            } else if let Some(_) = self.trail {
                self.trail_args_found += 1;
                return Some(Ok(Trail { value: string }));
            
            // No trail
            } else {
                self.finished = true;
                return Some(Err(UnexpectedPositional(string)));
            }
        }
        
        None
    }
}