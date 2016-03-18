use std::borrow::{Borrow};
use std::collections::{HashMap, HashSet};

/// A parsed argument.
// Lifetime 'a is the "definitions" and 'b is the "arguments".
#[derive(Debug, PartialEq)]
pub enum Arg<'a, 'b> {
    /// The name and value of a found positional.
    Positional(&'a str, &'b str),
    /// The value of a found trail argument (may occur multiple times).
    TrailPart(&'b str),
    /// The name of a found switch ("help" for ```--help```).
    Switch(&'a str),
    /// The name and value of an optional argument.
    Option(&'a str, &'b str),
}

/// The name of an optional argument.
#[derive(Debug, Clone)]
pub enum OptName<T: Borrow<str>> {
    /// ```--help```.
    Long(T),
    /// ```--help``` | ```-h```.
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
    /// A positional argument.
    Positional { name: T },
    /// A trail. The parameter is only used for help messages.
    Trail { trail: TrailType, parameter: Option<T> },
    /// A switch.
    Switch { name: OptName<T> },
    /// An option. The parameter is only used for help messages.
    Option { name: OptName<T>, parameter: Option<T> },
}

/// A partially specified argument definition.
#[must_use = "The argument definition is only partially constructed"]
pub struct PartialArgDef<T: Borrow<str>> {
    /// The name that was specified for this argument.
    pub name: OptName<T>
}

impl<T: Borrow<str>> PartialArgDef<T> {
    /// Creates a new ```switch``` definition.
    pub fn switch(self) -> ArgDef<T> {
        ArgDef::new(DefType::Switch { name: self.name })
    }
    
    /// Creates a new ```option``` definition.
    /// This is a type of flag that takes a single parameter.
    pub fn option(self) -> ArgDef<T> {
        ArgDef::new(DefType::Option { name: self.name, parameter: None })
    }
}

/// The definition of one or more arguments to expect when parsing.
#[derive(Debug, Clone)]
pub struct ArgDef<T: Borrow<str>> {
    /// The type/data of the definition.
    pub deftype: DefType<T>,
    /// An optional help string.
    pub help: Option<T>,
}

impl<T: Borrow<str>> ArgDef<T> {
    /// Creates a new argument definition.
    fn new(deftype: DefType<T>) -> ArgDef<T> {
        ArgDef {
            deftype: deftype,
            help: None,
        }
    } 
    
    /// Creates the definition for a positional argument with the given name.
    pub fn positional(name: T) -> ArgDef<T> {
        ArgDef::new(DefType::Positional { name: name })
    }
    
    /// Creates the definition for a trail of one or more arguments.
    pub fn required_trail() -> ArgDef<T> {
        ArgDef::new(DefType::Trail { trail: TrailType::OnePlus, parameter: None })
    }
    
    /// Creates the definition for a trail of zero or more arguments.
    pub fn optional_trail() -> ArgDef<T> {
        ArgDef::new(DefType::Trail{ trail: TrailType::ZeroPlus, parameter: None })
    }
    
    /// Starts creating a new optional argument with the given long name.
    /// This means that this definition is used when the name is given as an
    /// argument, prefixed with two dashes (eg. "help" => ```--help```).
    pub fn named(name: T) -> PartialArgDef<T> {
        PartialArgDef { name: OptName::Long(name) }
    }
    
    /// Starts creating a new optional argument with the given long name.
    /// This means that this definition is used when the name is given as an
    /// argument, prefixed with two dashes (eg. "help" => ```--help```).
    /// The short argument is for single-character abbreviations 
    /// (eg. "a" => ```-a```).
    pub fn named_and_short(name: T, short: char) -> PartialArgDef<T> {
        PartialArgDef { name: OptName::LongAndShort(name, short) }
    }
    
    /// Sets the help message for this argument.
    pub fn set_help(&mut self, help: T) -> &mut Self {
        self.help = Some(help);
        self
    }
    
    /// Sets the parameter name for this argument (used for help messages).
    pub fn set_parameter(&mut self, param: T) -> &mut Self {
        use self::DefType::*;
        match self.deftype {
            Trail { ref mut parameter, ..} | Option { ref mut parameter, ..} => {
                *parameter = Some(param);
            },
            _ => {}
        }
        self
    }
}

/// The types of argument trails to expect.
#[derive(Debug, Clone)]
pub enum TrailType {
    /// One or more arguments.
    OnePlus,
    /// Zero or more arguments.
    ZeroPlus,
}

/// An error found when defining the expected argument structure.
#[derive(Debug, PartialEq)]
pub enum DefinitionError<'a> {
    /// Two optional arguments have the same short name (eg: both ```--verbose```
    /// and ```--version``` using ```-v```).
    SameShortName(&'a str, &'a str),
    /// The optional name is defined twice.
    OptionDefinedTwice(&'a str),
    /// The positional name is defined twice.
    PositionalDefinedTwice(&'a str),
    /// Two trail definitions were given.
    TwoTrailsDefined,
}

/// The type of an optional argument.
#[derive(Debug, Clone)]
enum OptType {
    Switch,
    Option,
}

/// An error found when parsing.
#[derive(Debug, PartialEq)]
pub enum ParseError<'a, 'b> {
    /// The positional argument with this name was not found.
    MissingPositional(&'a str),
    /// The required parameter of this option was not found.
    MissingParameter(&'a str),
    /// No trail arguments were found, but they were expected.
    MissingTrail,
    /// No more positionals was expected, but this argument was found.
    UnexpectedPositional(&'b str),
    /// This short (```-h```) flag was not defined.
    UnexpectedShortArgument(char, &'b str),
    /// This long (```--help```) flag was not defined.
    UnexpectedLongArgument(&'b str),
    /// A short flag of an option taking a parameter was grouped with others.
    /// eg. ```-x ARG``` was instead grouped as ```-abcdx ARG```.
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
                    return Err(OptionDefinedTwice(name));
                } else {
                    used_names.insert(name);
                }
            },
            OptName::LongAndShort(ref long, short) => {
                let name = long.borrow();
                if used_names.contains(name) {
                    return Err(OptionDefinedTwice(name));
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
        let mut used_positional_names = HashSet::new();
        for def in expected {
            match def.deftype {
                Positional { ref name } => {
                    if used_positional_names.contains(name.borrow()) {
                        return Err(PositionalDefinedTwice(name.borrow()));
                    } else {
                        positional.push(name.borrow());
                        used_positional_names.insert(name.borrow());
                    }
                },
                Trail{ trail: ref trail_type, .. } => {
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
                Option { ref name, .. } => {
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
                Ok(Arg::Switch(name))
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
                            Ok(Arg::Option(name, string))
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
                return Some(Ok(Switch(name)));
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
                return Some(Ok(Positional(position, string)));
            
            // Trail
            } else if let Some(_) = self.trail {
                self.trail_args_found += 1;
                return Some(Ok(TrailPart(string)));
            
            // No trail
            } else {
                self.finished = true;
                return Some(Err(UnexpectedPositional(string)));
            }
        }
        
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn missing_positional() {
        let pos = ArgDef::positional("pos");
        let expected = &[pos];
        let args: Vec<String> = vec![];
        let mut parse = Parse::new(expected, &args).unwrap();
        assert_eq!(
            parse.next(), 
            Some(Err(ParseError::MissingPositional("pos")))
        );
    }
    
    #[test]
    fn example_parse_black_box() {
        use super::Arg::*;
        
        let a_foo = ArgDef::positional("foo");
        let a_foobar = ArgDef::required_trail();
        let a_help = ArgDef::named_and_short("help", 'h').switch();
        let a_version = ArgDef::named("version").switch();
        let a_verbose = ArgDef::named_and_short("verbose", 'v').switch();
        let a_exclude = ArgDef::named_and_short("exclude", 'x').option();
        let a_passed = ArgDef::named("").switch();
        
        let args = vec![
            String::from("foo"),
            String::from("bar"),
            String::from("-x"),
            String::from("baz"),
            String::from("--verbose"),
            String::from("--"),
            String::from("arg"),
            String::from("--help]"),
        ];
        
        let mut foo = "";
        let mut foobar = Vec::new();
        let mut verbose = false;
        let mut exclude = None;
        let mut passed = None;
        
        let expected = &[a_foo, a_foobar, a_help, a_version, a_verbose, a_exclude,
             a_passed];
    
        let mut parse = Parse::new(expected, &args).expect("Invalid definitions");
        while let Some(item) = parse.next() {
            match item {
                Ok(Positional { name: "foo", value }) => {
                    foo = value;
                },
                Ok(Trail { value }) => {
                    foobar.push(value);
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
        
        assert_eq!(foo, "foo");
        assert_eq!(foobar, vec!["bar"]);
        assert_eq!(exclude, Some("baz"));
        assert_eq!(verbose, true);
        assert_eq!(passed, Some(&args[6..]));    
    }
    
    #[test]
    fn example_parse_white_box() {
        use super::Arg::*;
        
        let a_foo = ArgDef::positional("foo");
        let a_foobar = ArgDef::required_trail();
        let a_help = ArgDef::named_and_short("help", 'h').switch();
        let a_version = ArgDef::named("version").switch();
        let a_verbose = ArgDef::named_and_short("verbose", 'v').switch();
        let a_exclude = ArgDef::named_and_short("exclude", 'x').option();
        let a_passed = ArgDef::named("").switch();
        
        let args = vec![
            String::from("foo"),
            String::from("bar"),
            String::from("-x"),
            String::from("baz"),
            String::from("--verbose"),
            String::from("--"),
            String::from("arg"),
            String::from("--version"),
            String::from("--help"),
        ];
        
        let expected = &[a_foo, a_foobar, a_help, a_version, a_verbose, a_exclude,
             a_passed];
    
        let mut parse = Parse::new(expected, &args).expect("Invalid definitions");
                
        assert_eq!(parse.next(), Some(Ok(Positional{ name: "foo", value: "foo"})));
        assert_eq!(parse.next(), Some(Ok(Trail { value: "bar"})));
        assert_eq!(parse.next(), Some(Ok(Option { name: "exclude", value: "baz" })));
        assert_eq!(parse.next(), Some(Ok(Switch { name: "verbose" })));
        assert_eq!(parse.next(), Some(Ok(Switch { name: "" })));
        assert_eq!(parse.next(), Some(Ok(Trail { value: "arg" })));
        assert_eq!(parse.next(), Some(Ok(Switch { name: "version" })));
        assert_eq!(parse.next(), Some(Ok(Switch { name: "help" })));
        assert_eq!(parse.next(), None);
    }
}