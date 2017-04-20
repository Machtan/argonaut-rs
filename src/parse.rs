use argdef::{SingleTarget, CollectionTarget, OptionTarget, ArgDef, ArgDefKind, SubCmd};
use help::Help;
use std::collections::{HashMap, HashSet, VecDeque};
use std::borrow::{Cow, Borrow};
use std::rc::Rc;

/// References to the targets of non-positional arguments.
//#[derive(Debug)]
pub enum TargetRef<'def, 'tar> {
    Flag(&'tar mut bool),
    Count(&'tar mut usize),
    OptArg(&'tar mut OptionTarget),
    Interrupt(Box<FnMut(Rc<Help<'def>>)>),
}

/// Sorted argument definitions. Updated mutably during the parse.
//#[derive(Debug)]
pub struct ParseState<'def, 'tar> {
    positional: VecDeque<(Cow<'def, str>, &'tar mut SingleTarget)>,
    // (satisfied, target)
    trail: Option<(Cow<'def, str>, bool, &'tar mut CollectionTarget)>,
    subcommands: HashMap<Cow<'def, str>, SubCmd<'def>>,
    options: HashMap<Cow<'def, str>, TargetRef<'def, 'tar>>,
    short_map: HashMap<Cow<'def, str>, Cow<'def, str>>,
}

impl<'def, 'tar> ParseState<'def, 'tar> {
    /// Returns the internal object representing the given option name.
    fn get_interned_name(&self, option: &str) -> Cow<'def, str> {
        self.options.keys().find(|k| k.as_ref() == option).unwrap().clone()
    }
    
    /// Attempts to find a target from the given option.
    fn get_target<'a>(&'a mut self, option: &str, help: Rc<Help<'def>>)
            -> Result<(Cow<'def, str>, &'a mut TargetRef<'def, 'tar>), ParseError<'def>> {
        let mut key = &option[2..];
        if ! option.starts_with("--") {
            if let Some(mapped_key) = self.short_map.get(&option[1..]) {
                key = mapped_key.as_ref();
            } else {
                return ParseError::parse(format!("Unknown option: '{}'", option), help);
            }
        }
        if ! self.options.contains_key(key) {
            return ParseError::parse(format!("Unknown option '{}'", option), help);
        }
        // INVARIANT: key is contained
        let name = self.get_interned_name(key);
        let target = self.options.get_mut(key).unwrap();
        Ok((name, target))
    }
    
    
    fn read_option<'arg, I>(&mut self, option: &str, args: &mut I, 
        given_values: &mut HashSet<Cow<'def, str>>, help: Rc<Help<'def>>) 
        -> Result<Option<Cow<'def, str>>, ParseError<'def>>
      where I: Iterator<Item=&'arg str>
    {
        use self::TargetRef::*;
        match self.get_target(option, help.clone())? {
            (_, &mut Flag(ref mut flag)) => {
                **flag = true;
            }
            (_, &mut Count(ref mut count)) => {
                **count += 1;
            }
            (ref name, &mut OptArg(ref mut value)) => {
                if given_values.contains(name) {
                    return ParseError::parse(format!("Option '{}' given twice!", name), help);
                }
                let arg = if let Some(arg) = args.next() {
                    arg
                } else {
                    return ParseError::parse(format!("Missing argument for option '{}'", option), help);
                };
                match value.parse(arg) {
                    Ok(_) => {}
                    Err(msg) => return ParseError::parse(msg, help),
                };
                given_values.insert(name.clone());
            }
            (ref name, &mut Interrupt(ref mut callback)) => {
                callback(help);
                return Ok(Some(name.clone()));
            }
        }
        Ok(None)
    }
}

fn validate_short<'def, N: AsRef<str>>(name: &N) -> Result<(), ParseError<'def>> {
    let name = name.as_ref();
    if name.starts_with("-") {
        ParseError::defs(format!("Invalid short identifier '{}'. Short ids may not start with '-'.", name))
    } else {
        Ok(())
    }
}

/// Sorts the given definitions and checks that all invariants are upheld.
pub fn parse_definitions<'def, 'tar>(defs: Vec<ArgDef<'def, 'tar>>) 
        -> Result<ParseState<'def, 'tar>, ParseError<'def>> {
    let mut positional = VecDeque::new();
    let mut trail = None;
    let mut options = HashMap::new(); // long-to-arg
    let mut short_map = HashMap::new(); // short-to-long
    let mut subcommands = HashMap::new();
    let mut has_positional = false;
    let mut has_subcommand = false;
    for def in defs {
        match def.kind {
            ArgDefKind::Positional { target } => {
                if has_subcommand {
                    return ParseError::defs(format!("Positional (+trail) and subcommand definitions cannot be used together."));
                }
                has_positional = true;
                positional.push_back((def.name, target));
            }
            ArgDefKind::Trail { optional, target } => {
                if has_subcommand {
                    return ParseError::defs(format!("Positional (+trail) and subcommand definitions cannot be used together."));
                }
                has_positional = true;
                if trail.is_some() {
                    return ParseError::defs(format!("Two trails defined."));
                }
                trail = Some((def.name, optional, target));
            }
            ArgDefKind::Subcommand { handler } => {
                if has_positional {
                    return ParseError::defs(format!("Positional (+trail) and subcommand definitions cannot be used together."));
                }
                has_subcommand = true;
                if subcommands.contains_key(&def.name) {
                    return ParseError::defs(format!("Sucommand '{}' defined twice", def.name))
                }
                subcommands.insert(def.name, handler);
            }
            ArgDefKind::Flag { short, target } => {
                if let Some(short) = short {
                    validate_short(&short)?;
                    if short_map.contains_key(&short) {
                        return ParseError::defs(format!("Short name '{}' defined twice.", short));
                    }
                    short_map.insert(short, def.name.clone());
                }
                if options.contains_key(&def.name) {
                    return ParseError::defs(format!("Option '{}' defined twice.", def.name));
                }
                options.insert(def.name, TargetRef::Flag(target));
            }
            ArgDefKind::Count { short, target } => {
                if let Some(short) = short {
                    validate_short(&short)?;
                    if short_map.contains_key(&short) {
                        return ParseError::defs(format!("Short name '{}' defined twice.", short));
                    }
                    short_map.insert(short, def.name.clone());
                }
                if options.contains_key(&def.name) {
                    return ParseError::defs(format!("Option '{}' defined twice.", def.name));
                }
                options.insert(def.name, TargetRef::Count(target));
            }
            ArgDefKind::OptArg { short, target } => {
                if let Some(short) = short {
                    validate_short(&short)?;
                    if short_map.contains_key(&short) {
                        return ParseError::defs(format!("Short name '{}' defined twice.", short));
                    }
                    short_map.insert(short, def.name.clone());
                }
                if options.contains_key(&def.name) {
                    return ParseError::defs(format!("Option '{}' defined twice.", def.name));
                }
                options.insert(def.name, TargetRef::OptArg(target));
            }
            ArgDefKind::Interrupt { short, callback } => {
                if let Some(short) = short {
                    validate_short(&short)?;
                    if short_map.contains_key(&short) {
                        return ParseError::defs(format!("Short name '{}' defined twice.", short));
                    }
                    short_map.insert(short, def.name.clone());
                }
                if options.contains_key(&def.name) {
                    return ParseError::defs(format!("Option '{}' defined twice.", def.name));
                }
                options.insert(def.name, TargetRef::Interrupt(callback));
            }
        }
    }
    Ok(ParseState { positional, trail, subcommands, options, short_map })
}

/// An error found when parsing arguments.
#[derive(Debug)]
pub enum ParseError<'def> {
    /// The given argument definitions aren't valid.
    InvalidDefinitions(String),
    
    /// The parse could not finish succesfully.
    ParseFailed(String, Rc<Help<'def>>),
    
    /// A subcommand failed to parse, and has been handled.
    SubParseFailed,
    
    /// An interrupt-flag with the given name was encountered.
    /// 
    /// The variables pointed to by the definitions will not all have been
    /// assigned their expected values.
    Interrupted(Cow<'def, str>)
}

impl<'def> ParseError<'def> {
    fn defs<T, S: Into<String>>(reason: S) -> Result<T, ParseError<'def>> {
        Err(ParseError::InvalidDefinitions(reason.into()))
    }
    
    fn parse<T, S: Into<String>>(reason: S, help: Rc<Help<'def>>) -> Result<T, ParseError<'def>> {
        Err(ParseError::ParseFailed(reason.into(), help))
    }
    
    fn interrupt<T>(name: Cow<'def, str>) -> Result<T, ParseError<'def>> {
       Err( ParseError::Interrupted(name))
    }
}

/// Parses the given arguments and updates the defined variables with them.
/// This version does not print usage in the case of parse errors, nor does 
/// it 'un-propagate' parsing errors.
pub fn parse_plain<'def, 'tar, T, P: Into<String>>(program: P, args: &[T], definitions: Vec<ArgDef<'def, 'tar>>) 
    -> Result<Option<i32>, ParseError<'def>>
  where T: Borrow<str> 
{ 
    let program = program.into();
    let help = Rc::new(Help::new(program.clone(), &definitions));
    let mut defs = parse_definitions(definitions)?;
    
    //println!("Defs: {:?}", defs);
    let mut args = args.iter().map(|e| e.borrow());
    
    // value-type definitions that have been given and should not be overridden
    let mut given_values = HashSet::new();
    
    while let Some(arg) = args.next() {
        // Option / interrupt
        if arg.starts_with("-") {
            if let Some(interrupt) = defs.read_option(arg, &mut args, &mut given_values, help.clone())? {
                return ParseError::interrupt(interrupt);
            }
        
        // Positional
        } else if ! defs.positional.is_empty() {
            let (_name, target) = defs.positional.pop_front().unwrap();
            match target.parse(arg) {
                Ok(()) => {},
                Err(msg) => return ParseError::parse(msg, help),
            } // MAYBE: chain err
        
        // Subcommand
        } else if ! defs.subcommands.is_empty() {
            if let Some(handler) = defs.subcommands.get_mut(arg) {
                let rest = args.collect::<Vec<_>>();
                let subprogram = format!("{} {}", program, arg);
                return handler(subprogram, &rest);
            } else {
                return ParseError::parse(format!("Unknown subcommand: '{}'", arg), help);
            }
        
        // Trail
        } else {
            if let Some((_, ref mut satisfied, ref mut target)) = defs.trail {
                match target.parse_and_add(arg) {
                    Ok(()) => {},
                    Err(msg) => return ParseError::parse(msg, help),
                }; // TODO: chain err
                *satisfied = true;
            } else {
                return ParseError::parse(format!("Unexpected argument '{}'", arg), help);
            }            
        }
    }
    
    if let Some((name, _)) = defs.positional.pop_front() {
        return ParseError::parse(format!("Missing positional argument '{}'", name), help);
    }
    
    if let Some((name, satisfied, _)) = defs.trail {
        if ! satisfied {
            return ParseError::parse(format!("Expected at least one trailing argument for '{}'", name), help);
        }
    }
    
    if ! defs.subcommands.is_empty() {
        return ParseError::parse(format!("No subcommand specified"), help);
    }
    
    Ok(None)
}

/// Parses the given arguments and updates the defined variables with them.
/// 
/// Errors are handled like this:
/// - Invalid argument definitions (logic error): Panic.
/// - Parse failed: Print usage and prevent the error from propagating.
/// - Interrupt or sub parse failed: Just passed along.
pub fn parse<'def, 'tar, T, P: Into<String>>(program: P, args: &[T], definitions: Vec<ArgDef<'def, 'tar>>) 
    -> Result<Option<i32>, ParseError<'def>>
  where T: Borrow<str> 
{ 
    match parse_plain(program, args, definitions) {
        Err(ParseError::InvalidDefinitions(msg)) => {
            panic!("Invalid definitions: {}", msg);
        }
        Err(ParseError::ParseFailed(msg, help)) => {
            println!("Parse failed: {}", msg);
            help.print_usage();
            Err(ParseError::SubParseFailed)
        }
        other => other,
    }
}
