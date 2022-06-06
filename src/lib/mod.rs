use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

mod log;
mod parser;
use parser::Code;
use parser::Parser;
mod err;

fn help_message(bin_path: &str) -> String {
    let msg = format!("Usage: {} <code path>", bin_path);
    let msg = format!("{}\n{}", msg, "use -v for more information");
    let msg = format!("{}\n{}", msg, "use -o <out file name> for output file name");

    return msg;
}

pub struct Target {
    code_file_path: String,
    execute_file_path: String,
    verbose: bool, // verbose mode -> debug mode
}

impl Target {
    pub fn new(args: &[String]) -> Result<Target, String> {
        if args.len() < 2 {
            return Err(help_message(args[0].as_str()).into());
        }
        let mut verbose = false;
        let mut execute_file_path = String::from("a.out");
        let code_file_path = args[args.len() - 1].clone();

        for i in 1..args.len() - 1 {
            if args[i] == "-v" {
                verbose = true;
            } else if args[i] == "-o" {
                if i + 1 >= args.len() - 1 {
                    return Err(help_message(args[0].as_str()).into());
                }
                let re = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
                if !re.is_match(&args[i + 1]) {
                    return Err(err::handler().e001());
                }
                execute_file_path = args[i + 1].clone();
                execute_file_path.push_str(".out")
            }
        }

        Ok(Target {
            code_file_path,
            execute_file_path,
            verbose,
        })
    }

    pub fn run(&self) -> Result<(), String> {
        println!("
       _                      _   _       _   _       _
__   _(_)_ __   ___ ___ _ __ | |_(_)_ __ | |_| |_ ___| |__
\\ \\ / / | '_ \\ / __/ _ \\ '_ \\| __| | '_ \\| __| __/ __| '_ \\
 \\ V /| | | | | (_|  __/ | | | |_| | | | | |_| |_\\__ \\ | | |
  \\_/ |_|_| |_|\\___\\___|_| |_|\\__|_|_| |_|\\__|\\__|___/_| |_|

                              _     _
  __ _ ___ ___  ___ _ __ ___ | |__ | | ___ _ __
 / _` / __/ __|/ _ \\ '_ ` _ \\| '_ \\| |/ _ \\ '__|
| (_| \\__ \\__ \\  __/ | | | | | |_) | |  __/ |
 \\__,_|___/___/\\___|_| |_| |_|_.__/|_|\\___|_|
    ");
        let code_file_path = Path::new(self.code_file_path.as_str());
        let user_code: String;
        let comma_regex = Regex::new(r"[ \t]*,[ \t]*").unwrap();
        let c_re = Regex::new(r"[ \t]+C[ \t]*'").unwrap();
        let x_re = Regex::new(r"[ \t]+X[ \t]*'").unwrap();

        match fs::read_to_string(code_file_path) {
            Ok(code) => user_code = code,
            Err(_) => {
                return Err(err::handler().e002());
            }
        }

        let mut parser = Parser::new(self.verbose);

        // user code line number
        let mut line_number: u32 = 1;
        // memory location
        let mut mem_loc: u32 = 0;
        let mut obj_code_list: Vec<Code> = Vec::new();
        let mut address_map: HashMap<u32, usize> = HashMap::new();
        let mut have_error = false;

        log::println("One pass:", self.verbose);

        for code in user_code.lines() {
            let source_code = code;
            let code = code.trim();
            let code = comma_regex.replace_all(code, ",");
            let code = c_re.replace_all(code.as_ref(), " C'");
            let code = x_re.replace_all(code.as_ref(), " X'");
            let code = code.as_ref();

            match parser.translate(line_number, mem_loc, code, source_code) {
                Ok((code, offset, need_modify_code)) => {
                    if parser.program_end && !code.no_obj_code {
                        log::println(
                            &format!("{}:\t{}\n-> ", line_number, source_code),
                            !self.verbose,
                        );
                        log::println(&err::handler().e304(), true);
                        break;
                    }
                    obj_code_list.push(code);
                    address_map.insert(mem_loc, obj_code_list.len() - 1);

                    for i in 0..need_modify_code.len() {
                        let code = &mut obj_code_list[address_map[&need_modify_code[i]]];
                        if let Err(e) = code.re_alloc(&mut parser) {
                            log::print(
                                &format!("{:X}:{}:\t{}\n-> ",need_modify_code[i], line_number, source_code),
                                !self.verbose,
                            );
                            log::println(&e, true);
                        }
                    }

                    log::println(&format!("move address {} ", offset), self.verbose);

                    mem_loc += offset;
                }
                Err(e) => {
                    if e == err::handler().e301() || e == err::handler().e306(){
                        io::stdout().flush().unwrap();
                        return Err(e);
                    }
                    have_error = true;
                    log::print(
                        &format!("{}:\t{}\n-> ", line_number, source_code),
                        !self.verbose,
                    );
                    log::println(&e, true);
                }
            }
            line_number += 1;
        }

        if !parser.program_end {
            return Err(err::handler().e304());
        }

        if self.verbose {
            println!("\n\nSymbolTable:");
            parser.symbols_inter();
            println!("\n\nObjectCode:");
            for i in 0..obj_code_list.len() {
                let code = &obj_code_list[i];
                if !code.no_obj_code {
                    let width = (code.byte * 2) as usize;
                    println!(
                        "{}:{} \n->0x{:04X} {:0width$X}",
                        code.line_number,
                        code.source_code,
                        code.location,
                        code.obj_code,
                        width = width,
                    );
                }
            }
        }

        // print binary code
        let mut start_address: u32;
        let program_start: u32;
        let program_name = parser.program_name.clone();
        match parser.get_symbol_location(program_name.as_str()) {
            Some((s, need_alloc)) => {
                if need_alloc {
                    return Err(err::handler().e309());
                }
                start_address = s;
                program_start = s;
            }
            None => {
                return Err(err::handler().e309());
            }
        }
        let mut contents = format!(
            "H^{:>6}{:06X}{:06X}\n",
            program_name,
            start_address,
            parser.program_length - start_address,
        );

        let mut now_bit = 4096;
        let mut relocation_bit: u16 = 0;
        let mut obj_code = String::new();
        let mut count = 0;

        for i in 0..obj_code_list.len() {
            let code = &obj_code_list[i];
            let width = (code.byte * 2) as usize;
            if code.undone {
                if code.base != "" {
                    print!("{}:\t{}\n-> ", code.line_number, code.source_code);
                    log::println(&err::handler().e312(&code.operand, &code.base), true);
                } else {
                    print!("{}:\t{}\n-> ", code.line_number, code.source_code);
                    log::println(&err::handler().e311(&code.operand), true);
                }
                have_error = true;
            }

            if (code.no_obj_code && width > 0) || (obj_code.len() - count) + width >= 60 {
                if obj_code.len() > 0 {
                    if program_start == 0x0 {
                        contents.push_str(&format!(
                            "T^{:06X}^{:02X}^{:03X}^{}\n",
                            start_address,
                            (obj_code.len() - count) / 2,
                            relocation_bit,
                            obj_code
                        ));
                    } else {
                        contents.push_str(&format!(
                            "T^{:06X}^{:03X}^{}\n",
                            start_address,
                            (obj_code.len() - count) / 2,
                            obj_code
                        ));
                    }
                }
                now_bit = 4096;
                relocation_bit = 0;
                start_address = code.location;
                obj_code = String::new();
                count = 0;
            }

            // RESW RESB
            if code.no_obj_code && width > 0 {
                let mut now = i + 1;

                while now < obj_code_list.len() {
                    let code = &obj_code_list[now];
                    if code.no_obj_code {
                        now = now + 1;
                    } else {
                        break;
                    }
                }
                if now < obj_code_list.len() {
                    start_address = (&obj_code_list[now]).location;
                }
                continue;
            }

            if !code.no_obj_code {
                if code.byte == 4 && !code.variable && code.ni != 1 {
                    relocation_bit += now_bit;
                }
                count += 1;
                obj_code.push_str(&format!("{:0width$X}^", code.obj_code, width = width));
                now_bit /= 2;
            }
        }

        if obj_code.len() > 0 {
            if parser.program_start_address == 0x0 {
                contents.push_str(&format!(
                    "T^{:06X}^{:02X}^{:03X}{}\n",
                    start_address,
                    (obj_code.len() - count) / 2,
                    relocation_bit,
                    obj_code
                ));
            } else {
                contents.push_str(&format!(
                    "T^{:06X}^{:03X}^{}\n",
                    start_address,
                    (obj_code.len() - count) / 2,
                    obj_code
                ));
            }
        }

        contents.push_str(&format!("E^{:06X}", parser.program_start_address));
        if !have_error {
            if let Err(e) = fs::write(&self.execute_file_path, contents){
                return Err(err::handler().e003(&e.to_string()));
            }
        }
        // END print binary code

        return Ok(());
    }
}
