use std::str::FromStr;
use std::fmt::Debug;
use std::borrow::Cow;
use std::rc::Rc;
use help::Help;
use parse::ParseError;
use std::collections::{BinaryHeap, BTreeSet, HashSet, LinkedList, VecDeque};
use std::hash::Hash;

pub type SubCmd<'def> = Box<FnMut(String, &[&str]) -> Result<Option<i32>, ParseError<'def>>>;

/// The description of an expected argument.
//#[derive(Debug)]
pub struct ArgDef<'def, 'tar> {
    pub(crate) name: Cow<'def, str>,
    pub(crate) kind: ArgDefKind<'def, 'tar>,
    pub(crate) help_desc: Option<Cow<'def, str>>,
}

//#[derive(Debug)]
pub(crate) enum ArgDefKind<'def, 'tar> {
    Positional { 
        target: &'tar mut SingleTarget,
    },
    Subcommand {
        handler: SubCmd<'def>,
    },
    Trail { 
        optional: bool, 
        target: &'tar mut CollectionTarget,
    },
    Flag {
        short: Option<Cow<'def, str>>,
        target: &'tar mut bool,
    },
    Count {
        short: Option<Cow<'def, str>>,
        target: &'tar mut usize,
    },
    OptArg {
        short: Option<Cow<'def, str>>,
        target: &'tar mut OptionTarget,
    },
    Interrupt {
        short: Option<Cow<'def, str>>,
        callback: Box<FnMut(Rc<Help<'def>>)>,
    },
}

// MAYBE: Make 'short'-setting safe somehow.
impl<'def, 'tar> ArgDef<'def, 'tar> {
    fn new<N>(name: N, kind: ArgDefKind<'def, 'tar>) -> ArgDef<'def, 'tar> 
      where N: Into<Cow<'def, str>> 
    {
        ArgDef {
            name: name.into(),
            kind: kind,
            help_desc: None,
        }
    }
    
    /// Creates a descrption of a required positional argument.
    ///
    /// The target value will be updated after the parse, as long as the parse 
    /// succeeds and is not interrupted by an `interrupt`-type argument.
    pub fn pos<N>(name: N, target: &'tar mut SingleTarget) -> ArgDef<'def, 'tar> 
      where N: Into<Cow<'def, str>> 
    {
        ArgDef::new(name, ArgDefKind::Positional { target })
    }
    
    /// Creates a description of a `trail`-type argument.
    ///
    /// The trail is a collection of the remaining positional arguments, after
    /// all defined ones have been passed. The trail can be set to be optional.
    pub fn trail<N>(name: N, optional: bool, target: &'tar mut CollectionTarget) -> ArgDef<'def, 'tar>
      where N: Into<Cow<'def, str>>
    {
        ArgDef::new(name, ArgDefKind::Trail { optional, target })
    }
    
    /// Creates a description of a subcommand.
    pub fn cmd<N, F>(name: N, handler: F) -> ArgDef<'def, 'tar>
      where N: Into<Cow<'def, str>>,
            F: 'static + FnMut(String, &[&str]) -> Result<Option<i32>, ParseError<'def>>
    {
        ArgDef::new(name, ArgDefKind::Subcommand { handler: Box::new(handler) })
    }
    
    /// Creates a description of an `interrupt`-type argument.
    ///
    /// When the identifier for this argument is passed, the callback is run,
    /// and the parsing is interrupted. This is for options that should interrupt
    /// the parse when encountered, such as `--help` and `--version`.
    pub fn interrupt<N, F>(name: N, callback: F) -> ArgDef<'def, 'tar>
      where N: Into<Cow<'def, str>>, F: FnMut(Rc<Help<'def>>) + 'static
    {
        ArgDef::new(name, ArgDefKind::Interrupt { 
            short: None, callback: Box::new(callback)
        })
    }
    
    /// Creates a description of an `option`-type argument.
    /// 
    /// If an argument is given, the target is set to Some(<parsed value>).
    pub fn option<N>(name: N, target: &'tar mut OptionTarget) -> ArgDef<'def, 'tar>
      where N: Into<Cow<'def, str>>
    {
        ArgDef::new(name, ArgDefKind::OptArg { short: None, target })
    }
    
    /// Creates a description of a `flag`-type argument.
    /// 
    /// This will set its target to true, when passed as an argument.
    pub fn flag<N>(name: N, target: &'tar mut bool) -> ArgDef<'def, 'tar>
      where N: Into<Cow<'def, str>>
    {
        ArgDef::new(name, ArgDefKind::Flag { short: None, target })
    }
    
    /// Creates a description of a `count`-type argument.
    /// 
    /// This will count the number of times the flag was passed in the arguments.
    pub fn count<N>(name: N, target: &'tar mut usize) -> ArgDef<'def, 'tar>
      where N: Into<Cow<'def, str>> 
    {
        ArgDef::new(name, ArgDefKind::Count { short: None, target })
    }
    
    /// Adds a short identifier for this option, like `-h` for `help`.
    ///
    /// **NOTE**: This method PANICS if used on a `positional`, `trail` or 
    /// `subcommand` description.
    ///
    /// # Example
    /// ```
    /// let mut eat_ice_cream = false;
    /// parse(&["-e"], &[
    ///     ArgDef::flag("eat_ice_cream", &mut eat_ice_cream).short("e"),
    /// ]).unwrap();
    /// assert_eq!(true, eat_ice_cream);
    /// ```
    pub fn short<N>(mut self, short: N) -> Self where N: Into<Cow<'def, str>> {
        use self::ArgDefKind::*;
        self.kind = match self.kind {
            Positional { .. } | Trail { .. } | Subcommand { .. } => {
                panic!("Positional, trail and subcommand arguments cannot have a short identifier");
            },
            Flag { target, .. } => Flag { short: Some(short.into()), target },
            Count { target, .. } => Count { short: Some(short.into()), target },
            OptArg { target, .. } => OptArg { short: Some(short.into()), target },
            Interrupt { callback, .. } => Interrupt { short: Some(short.into()), callback },
        };
        self
    }
    
    /// Adds a help description for this argument.
    pub fn help<N>(mut self, help: N) -> Self where N: Into<Cow<'def, str>> {
        self.help_desc = Some(help.into());
        self
    }
}

/// Allows every type that is FromStr to be read from an argument.
pub trait SingleTarget: Debug {
    /// Parses the value and updates self with it.
    fn parse(&mut self, value: &str) -> Result<(), String>;
}

impl<T> SingleTarget for T where T: Debug + FromStr {
    // TODO: Better info here.
    fn parse(&mut self, value: &str) -> Result<(), String> {
        let value = match <T as FromStr>::from_str(value) {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not parse and convert '{}'", value)),
        };
        *self = value;
        Ok(())
    }
}

/// Allows every type that is FromStr to be read from an argument.
pub trait OptionTarget: Debug {
    /// Parses the value and updates self with it.
    fn parse(&mut self, value: &str) -> Result<(), String>;
}

impl<T> OptionTarget for Option<T> where T: Debug + FromStr {
    // TODO: Better info here.
    fn parse(&mut self, value: &str) -> Result<(), String> {
        let value = match <T as FromStr>::from_str(value) {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not parse and convert '{}'", value)),
        };
        *self = Some(value);
        Ok(())
    }
}

/// Allows a collection to be extended with values read from arguments.
pub trait CollectionTarget: Debug {
    /// Parses the value and adds it to this collection.
    fn parse_and_add(&mut self, value: &str) -> Result<(), String>;
}

impl<T> CollectionTarget for Vec<T> where T: Debug + FromStr {
    fn parse_and_add(&mut self, value: &str) -> Result<(), String> {
        let value = match <T as FromStr>::from_str(value) {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not parse and convert '{}'", value)),
        };
        self.push(value);
        Ok(())
    }
}

impl<T> CollectionTarget for BinaryHeap<T> where T: Debug + FromStr + Ord {
    fn parse_and_add(&mut self, value: &str) -> Result<(), String> {
        let value = match <T as FromStr>::from_str(value) {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not parse and convert '{}'", value)),
        };
        self.push(value);
        Ok(())
    }
}

impl<T> CollectionTarget for BTreeSet<T> where T: Debug + FromStr + Ord {
    fn parse_and_add(&mut self, value: &str) -> Result<(), String> {
        let value = match <T as FromStr>::from_str(value) {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not parse and convert '{}'", value)),
        };
        self.insert(value);
        Ok(())
    }
}

impl<T> CollectionTarget for HashSet<T> where T: Debug + FromStr + Hash + Eq {
    fn parse_and_add(&mut self, value: &str) -> Result<(), String> {
        let value = match <T as FromStr>::from_str(value) {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not parse and convert '{}'", value)),
        };
        self.insert(value);
        Ok(())
    }
}

impl<T> CollectionTarget for LinkedList<T> where T: Debug + FromStr {
    fn parse_and_add(&mut self, value: &str) -> Result<(), String> {
        let value = match <T as FromStr>::from_str(value) {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not parse and convert '{}'", value)),
        };
        self.push_back(value);
        Ok(())
    }
}

impl<T> CollectionTarget for VecDeque<T> where T: Debug + FromStr {
    fn parse_and_add(&mut self, value: &str) -> Result<(), String> {
        let value = match <T as FromStr>::from_str(value) {
            Ok(val) => val,
            Err(_) => return Err(format!("Could not parse and convert '{}'", value)),
        };
        self.push_back(value);
        Ok(())
    }
}
