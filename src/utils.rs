//! Outdated!

fn align_lines(lines: &mut Vec<Vec<String>>, padding: Option<char>) {
    let mut widths = Vec::new();
    // Calculate widths
    for line in lines.iter() {
        while line.len() > widths.len() {
            widths.push(0);
        }
        for (i, item) in line.iter().enumerate() {
            if item.len() > widths[i] {
                widths[i] = item.len();
            }
        }
    }
    
    let padding = if let Some(pad) = padding {
        pad
    } else {
        ' '
    };
    // Pad the lines
    for line in lines.iter_mut() {
        if line.is_empty() { continue; }
        for (i, item) in line.iter_mut().enumerate() {
            let target_width = widths[i];
            while item.len() < target_width {
                item.push(padding);
            }
        }
    }
}

/// Generates a help message for the tool based on the given list of arguments,
/// their parameter name (if relevant), and their help string.
pub fn generate_help<'a>(parser: &Parser<'a>) -> String {
    use arg::ArgType::*;
    use common::OptName::*;
    
    let args = internal_get_definitions(parser);
    let mut help_message = String::new();
    
    let mut required = Vec::new();
    let mut interrupting = Vec::new();
    let mut passing = Vec::new();
    let mut optional = Vec::new();
    for (i, &arg) in args.iter().enumerate() {
        let argtype = arg::internal_get_raw(arg);
        match argtype {
            Single(_) | ZeroPlus(_) | OnePlus(_) => {
                required.push((i, argtype));
            },
            Interrupt(_) => {
                interrupting.push((i, argtype));
            },
            OptSingle(_) | OptZeroPlus(_) | OptOnePlus(_) | Switch(_) => {
                optional.push((i, argtype));
            },
            PassAlong(_) => {
                passing.push((i, argtype));
            }
        }
    }
    
    if ! required.is_empty() {
        help_message.push_str("Required arguments:\n");
    
        let mut lines = Vec::new();
        let mut help_texts = Vec::new();
        for (i, argtype) in required {
            match argtype {
                Single(name) => {
                    lines.push(vec![name.to_owned()]);
                },
                OnePlus(name) => {
                    lines.push(vec![format!("{0} [{0}, ..]", name)]);
                },
                ZeroPlus(name) => {
                    lines.push(vec![format!("[{}, ..]", name)]);
                },
                _ => unreachable!(),
            }
            help_texts.push(args[i].help());
        }
        align_lines(&mut lines, None);
        for (i, line) in lines.iter().enumerate() {
            help_message.push_str("  ");
            for part in line {
                help_message.push_str(part);
                help_message.push(' ');
            }
            help_message.push_str("   ");
            help_message.push_str(help_texts[i]);
            help_message.push_str("\n");
        }
    }
    
    if ! interrupting.is_empty() {
        if ! help_message.is_empty() {
            help_message.push_str("\n");
        }
        help_message.push_str("Interrupts:\n");
        let mut lines = Vec::new();
        let mut help_texts = Vec::new();
        for (i, argtype) in interrupting {
            match argtype {
                Interrupt(Normal(long)) => {
                    lines.push(vec![format!("--{}", long)]);
                },
                Interrupt(NormalAndShort(long, short)) => {
                    lines.push(vec![
                        format!("--{}", long),
                        "|".to_owned(),
                        format!("-{}", short)
                    ]);
                },
                _ => unreachable!(),
            };
            help_texts.push(args[i].help());
        }
        
        align_lines(&mut lines, None);
        let mut combined = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let mut text = String::new();
            for part in line {
                text.push_str(part);
                text.push(' ');
            }
            combined.push(vec![text, help_texts[i].to_owned()]);
        }
        align_lines(&mut combined, None);

        for line in combined {
            help_message.push_str("  ");
            help_message.push_str(&line[0]);
            help_message.push_str("   ");
            help_message.push_str(&line[1]);
            help_message.push_str("\n");
        }
    }
    
    if ! optional.is_empty() {
        // Add a separating space
        if ! help_message.is_empty() {
            help_message.push_str("\n");
        }
        help_message.push_str("Optional arguments:\n");
    
        let mut lines = Vec::new();
        let mut help_texts = Vec::new();
        for (i, argtype) in optional {
            let mut param = args[i].param().to_owned();
            if param.is_empty() {
                param = args[i].name().to_uppercase();
            }
            match argtype {
                OptSingle(Normal(long)) => {
                    lines.push(vec![
                        format!("--{}{}", long, param)
                    ]);
                },
                OptSingle(NormalAndShort(long, short)) => {
                    lines.push(vec![
                        format!("--{}", long),
                        "|".to_owned(),
                        format!("-{}", short),
                        param
                    ]);
                },
                OptZeroPlus(Normal(long)) => {
                    lines.push(vec![
                        format!("--{}[{}, ..]", long, param)
                    ]);
                },
                OptZeroPlus(NormalAndShort(long, short)) => {
                    lines.push(vec![
                        format!("--{}", long),
                        "|".to_owned(),
                        format!("-{}", short),
                        format!("[{}, ..]", param)
                    ]);
                },
                OptOnePlus(Normal(long)) => {
                    lines.push(vec![
                        format!("--{0} {1} [{1}, ..]", long, param),
                    ]);
                },
                OptOnePlus(NormalAndShort(long, short)) => {
                    lines.push(vec![
                        format!("--{}", long), 
                        "|".to_owned(),
                        format!("-{}", short),
                        format!("{0} [{0}, ..]", param)
                    ]);
                },
                Switch(Normal(long)) => {
                    lines.push(vec![format!("--{}", long)]);
                },
                Switch(NormalAndShort(long, short)) => {
                    lines.push(vec![
                        format!("--{}", long), 
                        "|".to_owned(),
                        format!("-{}", short)
                    ]);
                },
                _ => unreachable!(),
            };
            help_texts.push(args[i].help());
        }

        align_lines(&mut lines, None);
        let mut combined = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let mut text = String::new();
            for part in line {
                text.push_str(part);
                text.push(' ');
            }
            combined.push(vec![text, help_texts[i].to_owned()]);
        }
        align_lines(&mut combined, None);

        for line in combined {
            help_message.push_str("  ");
            help_message.push_str(&line[0]);
            help_message.push_str("   ");
            help_message.push_str(&line[1]);
            help_message.push_str("\n");
        }
    }
    
    if ! passing.is_empty() {
        // Add a separating space
        if ! help_message.is_empty() {
            help_message.push_str("\n");
        }
        help_message.push_str("Pass-alongs:\n");
    
        let mut lines = Vec::new();
        let mut help_texts = Vec::new();
        for (i, argtype) in passing {
            let mut param = args[i].param().to_owned();
            if param.is_empty() {
                param = args[i].name().to_uppercase();
            }
            match argtype {
                PassAlong(Normal(long)) => {
                    lines.push(vec![
                        format!("--{}{}...", long, param)
                    ]);
                },
                PassAlong(NormalAndShort(long, short)) => {
                    lines.push(vec![
                        format!("--{}", long), 
                        "|".to_owned(),
                        format!("-{}", short),
                        format!("{}...", param)
                    ]);
                },
                _ => unreachable!(),
            }
            help_texts.push(args[i].help());
        }
        
        align_lines(&mut lines, None);
        let mut combined = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            let mut text = String::new();
            for part in line {
                text.push_str(part);
                text.push(' ');
            }
            combined.push(vec![text, help_texts[i].to_owned()]);
        }
        align_lines(&mut combined, None);

        for line in combined {
            help_message.push_str("  ");
            help_message.push_str(&line[0]);
            help_message.push_str("   ");
            help_message.push_str(&line[1]);
            help_message.push_str("\n");
        }
    }
    if help_message.ends_with("\n") {
        help_message.pop();
    }
    help_message
}