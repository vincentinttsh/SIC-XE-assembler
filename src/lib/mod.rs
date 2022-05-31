use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{self, Write};

mod parser;
use parser::Code;
use parser::Parser;

fn help_message(bin_path: &str) -> String {
    let msg = format!("Usage: {} <code path>", bin_path);
    let msg = format!("{}\n{}", msg, "use -v for more information");
    let msg = format!("{}\n{}", msg, "use -o <out file name> for output file name");

    return msg;
}

pub struct Target {
    code_file_path: String,
    execute_file: String,
    verbose: bool,
}

impl Target {
    pub fn new(args: &[String]) -> Result<Target, Box<dyn Error>> {
        if args.len() < 2 {
            return Err(help_message(args[0].as_str()).into());
        }
        let mut verbose = false;
        let mut execute_file = String::from("a.out");

        for i in 1..args.len() - 1 {
            if args[i] == "-v" {
                verbose = true;
            } else if args[i] == "-o" {
                if i + 1 >= args.len() - 1 {
                    return Err(help_message(args[0].as_str()).into());
                }
                let re = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
                if !re.is_match(&args[i + 1]) {
                    return Err(format!("invalid file name: {}", args[i + 1]).into());
                }
                execute_file = args[i + 1].clone();
                execute_file.push_str(".out")
            }
        }

        let code_file_path = args[args.len() - 1].clone();

        Ok(Target {
            code_file_path,
            execute_file: String::from(execute_file),
            verbose,
        })
    }
}

pub fn run(target: &Target) -> Result<(), Box<dyn Error>> {
    let user_code = fs::read_to_string(target.code_file_path.clone())?;
    let re = Regex::new(r",[ \t]+")?;
    let c_re = Regex::new(r"[ \t]+C[ \t]+")?;
    let x_re = Regex::new(r"[ \t]+X[ \t]+")?;

    let mut parser = Parser::new(target.verbose)?;
    let mut line_number: u32 = 1;
    let mut address: u16 = 0x0;
    let mut byte_code_list: Vec<Code> = Vec::new();
    let mut address_map: HashMap<u16, usize> = HashMap::new();
    let mut have_error = false;

    if target.verbose {
        println!("{}", "One pass:");
    }

    for line in user_code.lines() {
        let original_line = line;
        let line = re.replace_all(line, ",").clone();
        let line = line.trim();
        let line = c_re.replace_all(line, " C").clone();
        let line = line.trim();
        let line = x_re.replace_all(line, " X").clone();

        match parser.translate(address, line.clone().trim(), line_number, original_line) {
            Ok((code, move_address, need_modify_obj_code)) => {
                if parser.program_end && !code.nocode.get() {
                    if !target.verbose {
                        print!("{}:\t{}\n-> ", line_number, original_line);
                    }
                    println!("{}", "Error: have code after END");
                    break;
                }
                byte_code_list.push(code);
                address_map.insert(address, byte_code_list.len() - 1);

                for i in 0..need_modify_obj_code.len() {
                    match address_map.get(&need_modify_obj_code[i]) {
                        Some(line_num) => {
                            let code = &byte_code_list[*line_num];
                            if let Err(e) = code.re_alloc(&mut parser) {
                                if !target.verbose {
                                    print!("{}:\t{}\n-> ", line_number, original_line);
                                }
                                println!("{}", e);
                            }
                        }
                        None => {
                            return Err(format!(
                                "assembler error: line {}: address {} not found",
                                line_number, need_modify_obj_code[i]
                            )
                            .into());
                        }
                    }
                }

                address += move_address;
            }
            Err(e) => {
                if e.to_string() == "code need to start with a legal START mnemonic" {
                    io::stdout().flush().unwrap();
                    return Err(e.into());
                }
                have_error = true;
                if !target.verbose {
                    print!("{}:\t{}\n-> ", line_number, original_line);
                }
                println!("{}", e);
            }
        }
        line_number = line_number + 1;
    }

    if !parser.program_end {
        return Err("code need to have END mnemonic".into());
    }

    if target.verbose {
        println!("\n\nSymbolTable:");
        parser.symbols_inter();
        println!("\n\nObjectCode:");
        for i in 0..byte_code_list.len() {
            let code = &byte_code_list[i];
            if !code.nocode.get() {
                let width = (code.byte.get() * 2) as usize;
                println!(
                    "{}:{} \n->0x{:04X} {:0width$X}",
                    code.line_number,
                    code.code,
                    code.address.get(),
                    code.obj_code.get(),
                    width = width,
                );
            }
        }
    }

    if have_error {
        return Ok(());
    }

    // print binary code
    let mut contents = format!(
        "H{:>6}{:06X}{:06X}\n",
        parser.program_name,
        parser.program_start_address,
        parser.program_length.get(),
    );

    let mut now_bit = 4096;
    let mut relocation_bit: u16 = 0;
    let mut start_address = parser.program_start_address;
    let mut obj_code = String::new();

    for i in 0..byte_code_list.len() {
        let code = &byte_code_list[i];
        let width = (code.byte.get() * 2) as usize;
        if code.undone.get() {
            if code.base.borrow().to_string() != "" {
                print!("{}:\t{}\n-> ", code.line_number, code.code);
                println!(
                    "operand {} or {} not found",
                    code.operand.borrow(),
                    code.base.borrow()
                );
            } else {
                print!("{}:\t{}\n-> ", code.line_number, code.code);
                println!(
                    "operand {} or {} not found",
                    code.operand.borrow(),
                    code.base.borrow()
                );
            }
            have_error = true;
        }

        if (code.nocode.get() && width > 0) || obj_code.len() + width >= 60 {
            // println!("len: {}, width: {}", obj_code.len(), width);
            if obj_code.len() > 0 {
                if parser.program_start_address == 0x0 {
                    contents.push_str(&format!(
                        "T{:06X}{:02X}{:03X}{}\n",
                        start_address,
                        obj_code.len(),
                        relocation_bit,
                        obj_code
                    ));
                } else {
                    contents.push_str(&format!(
                        "T{:06X}{:03X}{}\n",
                        start_address,
                        obj_code.len(),
                        obj_code
                    ));
                }
            }
            now_bit = 4096;
            relocation_bit = 0;
            start_address = code.address.get();
            obj_code = String::new();
        }

        // RESW RESB
        if code.nocode.get() && width > 0 {
            let mut now = i + 1;

            while now < byte_code_list.len() {
                let code = &byte_code_list[now];
                if code.nocode.get() {
                    now = now + 1;
                } else {
                    break;
                }
            }
            if now < byte_code_list.len() {
                start_address = (&byte_code_list[now]).address.get();
            }
            continue;
        }

        if !code.nocode.get() {
            if code.byte.get() == 4 && !code.variable.get() {
                relocation_bit += now_bit;
            }
            obj_code.push_str(&format!("{:0width$X}", code.obj_code.get(), width = width));
            now_bit /= 2;
        }
    }

    if obj_code.len() > 0 {
        if parser.program_start_address == 0x0 {
            contents.push_str(&format!(
                "T{:06X}{:02X}{:03X}{}\n",
                start_address,
                obj_code.len(),
                relocation_bit,
                obj_code
            ));
        } else {
            contents.push_str(&format!(
                "T{:06X}{:03X}{}\n",
                start_address,
                obj_code.len(),
                obj_code
            ));
        }
    }

    contents.push_str(&format!("E{:06X}", parser.program_start_address));
    if !have_error {
        fs::write(&target.execute_file, contents)?;
    }
    // END print binary code

    Ok(())
}
