use std::borrow::Cow;
use argdef::{ArgDef, ArgDefKind};
use std_unicode::str::UnicodeStr;

pub fn trim_and_strip_lines<'a>(text: &'a str) -> impl Iterator<Item=&'a str> {
    let rev: Vec<_> = text.lines()
        .rev().skip_while(|&l| l == "" || l.is_whitespace()).collect();
    rev.into_iter()
        .rev().skip_while(|&l| l == "" || l.is_whitespace())
        .map(|line| line.trim())
}

fn write_trimmed_n<'def, T: AsRef<str>>(s: &mut String, prefix: &str, text: T) {
    for line in trim_and_strip_lines(text.as_ref()) {
        s.push_str(prefix);
        s.push_str(line);
        s.push('\n')
    }
}


/// A collection of descriptions of the defined arguments.
#[derive(Debug)]
pub struct Help<'def> {
    /// The 'command path' of the run program, eg. `cargo` or `cargo new`.
    pub program: String,
    /// Positional arguments.
    pub positional: Vec<(Cow<'def, str>, Option<Cow<'def, str>>)>,
    /// Trailing positional vararg.
    pub trail: Option<(Cow<'def, str>, bool, Option<Cow<'def, str>>)>,
    /// Subcommand arguments.
    pub subcommands: Vec<(Cow<'def, str>, Option<Cow<'def, str>>)>,
    /// Optional arguments.
    pub options: Vec<(Cow<'def, str>, Option<Cow<'def, str>>, HelpOptKind, Option<Cow<'def, str>>)>,
    /// Is `--help` defined.
    pub help_defined: bool,
}

impl<'def> Help<'def> {
    /// Creates a new help object from the given descriptions.
    pub fn new<'tar>(program: String, definitions: &[ArgDef<'def, 'tar>]) -> Help<'def> {
        let mut positional = Vec::new();
        let mut trail = None;
        let mut options = Vec::new();
        let mut subcommands = Vec::new();
        let mut help_defined = false;
        for def in definitions {
            match def.kind {
                ArgDefKind::Positional { .. } => {
                    positional.push((def.name.clone(), def.help_desc.clone()));
                }
                ArgDefKind::Trail { optional, .. } => {
                    trail = Some((def.name.clone(), optional, def.help_desc.clone()));
                },
                ArgDefKind::Subcommand { .. } => {
                    subcommands.push((def.name.clone(), def.help_desc.clone()));
                }
                ArgDefKind::Flag { ref short, .. } => {
                    options.push((
                        def.name.clone(), short.clone(), 
                        HelpOptKind::Flag, def.help_desc.clone()
                    ));
                }
                ArgDefKind::Count { ref short, .. } => {
                    options.push((
                        def.name.clone(), short.clone(), 
                        HelpOptKind::Count, def.help_desc.clone()
                    ));
                }
                ArgDefKind::OptArg { ref short, .. } => {
                    options.push((
                        def.name.clone(), short.clone(), 
                        HelpOptKind::OptArg, def.help_desc.clone()
                    ));
                }
                ArgDefKind::Interrupt { ref short, .. } => {
                    if def.name.as_ref() == "help" {
                        help_defined = true;
                    }
                    options.push((
                        def.name.clone(), short.clone(), 
                        HelpOptKind::Interrupt, def.help_desc.clone()
                    ));
                }
            }
        }
        Help { program, positional, trail, subcommands, options, help_defined }
    }
    
    fn write_usage_into(&self, s: &mut String) {
        s.push_str(&self.program);
        
        if ! self.options.is_empty() {
            if self.help_defined {
                if self.options.len() > 1 { // Not only --help
                    s.push_str(" [ --help | OPTIONS ]");
                } else {
                    s.push_str(" [ --help ]");
                }
            } else {
                s.push_str(" [ OPTIONS ]");
            }
        }
        
        for &(ref name, _) in self.positional.iter() {
            s.push(' ');
            s.push_str(name.as_ref());
        }
        
        if let Some((ref name, optional, _)) = self.trail {
            s.push(' ');
            if optional {
                s.push_str(&format!("[{}...]", name));
            } else {
                s.push_str(&format!("{} [{}...]", name, name));
            }
        }
        
        if self.subcommands.len() == 1 {
            s.push(' ');
            let ref name = self.subcommands[0].0;
            s.push_str(name.as_ref());
            s.push_str(" [args...]");
        } else if self.subcommands.len() > 1 {
            s.push_str(" { ");
            let last = self.subcommands.len() - 1;
            for (i, &(ref name, _)) in self.subcommands.iter().enumerate() {
                s.push_str(name.as_ref());
                if i != last {
                    s.push_str(" | ");
                }
            }
            s.push_str(" } [args...]");
        }
    }
    
    /// Generates a usage message for this program.
    pub fn usage_message(&self) -> String {
        let mut s = String::new();
        self.write_usage_into(&mut s);
        s
    }
    
    /// Prints a usage message for this program.
    pub fn print_usage(&self) {
        println!("Usage: {}", self.usage_message());
    }
    
    /// Generates a help message for this program, using the given program
    /// description. The description may be left blank.
    pub fn help_message(&self, description: &str) -> String {
        let mut s = String::from("Usage:\n  ");
        self.write_usage_into(&mut s);
        
        let has_description = description != "";
        let has_positional = (! self.positional.is_empty()) || self.trail.is_some();
        let has_optional = ! self.options.is_empty();
        let has_subcommands = ! self.subcommands.is_empty();
        if has_positional || has_optional || has_description || has_subcommands {
            s.push_str("\n\n");
        }
        
        if has_description {
            s.push_str("Description:\n");
            write_trimmed_n(&mut s, "  ", description);
        }
        
        if has_positional {
            s.push('\n');
            s.push_str("Positional arguments:\n");
            for &(ref name, ref help) in self.positional.iter() {
                s.push_str(&format!("  {}\n", name));
                if let &Some(ref help) = help {
                    write_trimmed_n(&mut s, "    ", help);
                }
                s.push('\n');
            }
            if let Some((ref name, optional, ref help)) = self.trail {
                s.push_str("  ");
                if optional {
                    s.push_str(&format!("[{}...]\n", name));
                } else {
                    s.push_str(&format!("{} [{}...]\n", name, name));
                }
                if let &Some(ref help) = help {
                    write_trimmed_n(&mut s, "    ", help);
                }
                s.push('\n');
            }
        }
        
        if has_subcommands {
            s.push('\n');
            s.push_str("Subcommands:\n");
            for &(ref name, ref help) in self.subcommands.iter() {
                s.push_str(&format!("  {}\n", name));
                if let &Some(ref help) = help {
                    write_trimmed_n(&mut s, "    ", help);
                }
                s.push('\n');
            }
        }
        
        if has_optional {
            if ! (has_positional || has_subcommands) {
                s.push('\n');
            }
            s.push_str("Optional arguments:\n");
            for &(ref name, ref short, kind, ref help) in self.options.iter() {
                s.push_str("  ");
                s.push_str("--");
                s.push_str(name.as_ref());
                if let &Some(ref short) = short {
                    s.push_str(", ");
                    s.push('-');
                    s.push_str(short.as_ref());
                }
                match kind {
                    HelpOptKind::OptArg => {
                        s.push(' ');
                        s.push_str(&name.as_ref().to_uppercase());
                    }
                    _ => {}
                }
                s.push('\n');
                if let &Some(ref help) = help {
                    write_trimmed_n(&mut s, "      ", help);
                    s.push('\n');
                }
            }
        }
        
        s
    }
    
    /// Prints a help message for this program, using the given program
    /// description. The description may be left blank.
    pub fn print_help(&self, description: &str) {
        print!("{}", self.help_message(description));
    }
}

/// Describes what kind of argument is expected.
#[derive(Debug, Clone, Copy)]
pub enum HelpOptKind {
    /// A flag. `./bin --verbose` => `true`
    Flag,
    /// A count. `./bin -v -v -v -v` => `4`
    Count,
    /// An option with a value. `./bin --eat-cake yes`
    OptArg,
    /// An interrupt. `./bin --help`
    Interrupt,
}
