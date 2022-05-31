use regex::Regex;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;

mod opcode_table;
use opcode_table::OpcodeTable;
mod symbol_table;
use symbol_table::SymbolTable;

pub struct Parser {
    opcode_table: OpcodeTable,
    symbol_table: SymbolTable,
    separator: Regex,
    comma_separator: Regex,
    program_start: bool,
    pub program_start_address: u16,
    pub program_end: bool,
    pub program_name: String,
    pub program_length: Cell<u16>,
    wait_for_base: Cell<bool>,
    reserve: HashMap<String, ()>,
    registers: HashMap<String, u8>,
    base: RefCell<String>,
    verbose: bool,
}

pub struct Code {
    pub obj_code: Cell<u32>,
    pub address: Cell<u16>,
    pub byte: Cell<u16>,
    ni: Cell<u32>,
    pub variable: Cell<bool>,
    pub nocode: Cell<bool>,
    pub comment: String,
    pub code: String,
    pub line_number: u32,
    pub undone: Cell<bool>,
    pub base: RefCell<String>,
    pub operand: RefCell<String>,
}

impl Code {
    pub fn new(user_code: &str, comment: &str, line_number: u32) -> Code {
        Code {
            obj_code: Cell::new(0x0),
            address: Cell::new(0x0),
            byte: Cell::new(0x0),
            ni: Cell::new(0x0),
            variable: Cell::new(false),
            nocode: Cell::new(false),
            comment: comment.to_string(),
            code: user_code.to_string(),
            line_number: line_number,
            undone: Cell::new(false),
            base: RefCell::new(String::new()),
            operand: RefCell::new(String::new()),
        }
    }
    fn xbpe(&self) -> u8 {
        if self.byte.get() > 2 {
            let obj_code = format!(
                "{:0width$X}",
                self.obj_code.get(),
                width = self.byte.get() as usize
            )
            .chars()
            .nth(2)
            .unwrap();

            let xbpe = u8::from_str_radix(&obj_code.to_string(), 16).unwrap();

            return xbpe;
        }
        return 0;
    }
    pub fn re_alloc(&self, parser: &mut Parser) -> Result<(), Box<dyn Error>> {
        let pc: i16 = (self.address.get() + self.byte.get()) as i16;
        let operand_byte_code: u16;

        (operand_byte_code, _) = parser.symbol(&self.operand.borrow());
        if self.xbpe() % 2 == 1 {
            self.obj_code
                .set(self.obj_code.get() + operand_byte_code as u32);
            self.undone.set(false);
        } else if self.ni.get() == 1 {
            self.obj_code
                .set(self.obj_code.get() + operand_byte_code as u32);
            self.undone.set(false);
        } else {
            let (operand, need_alloc, xbpe) = parser.fill_operand_base(
                self.address.get(),
                pc,
                &self.base.borrow(),
                operand_byte_code,
            )?;
            if !need_alloc {
                let xbpe = xbpe + self.xbpe();
                let operand_str: String =
                    format!("{:0width$X}", operand, width = self.byte.get() as usize);
                let mut operand = String::new();

                for i in operand_str.len() - 3..operand_str.len() {
                    operand.push(operand_str.chars().nth(i).unwrap());
                }
                let mut operand_byte_code = u32::from_str_radix(&operand, 16)?;
                if self.byte.get() == 4 {
                    operand_byte_code = operand_byte_code + ((xbpe as u32) << 20);
                } else {
                    operand_byte_code = operand_byte_code + ((xbpe as u32) << 12);
                }

                self.obj_code.set(self.obj_code.get() + operand_byte_code);
                self.undone.set(false);
            }
        }

        return Ok(());
    }
}

impl Parser {
    const IMMEDIATE_ADDRESS: u32 = 0b01;
    const INDIRECT_ADDRESS: u32 = 0b10;
    const SIMPLE_ADDRESS: u32 = 0b11;

    pub fn new(verbose: bool) -> Result<Parser, Box<dyn Error>> {
        Ok(Parser {
            opcode_table: OpcodeTable::new()?,
            symbol_table: SymbolTable::new(),
            separator: Regex::new(r"[ \t]+")?,
            comma_separator: Regex::new(r",")?,
            program_start: false,
            program_start_address: 0x0,
            program_end: false,
            program_name: String::new(),
            program_length: Cell::new(0),
            base: RefCell::new(String::new()),
            verbose: verbose,
            wait_for_base: Cell::new(false),
            reserve: HashMap::from([
                ("WORD".to_string(), ()),
                ("BYTE".to_string(), ()),
                ("START".to_string(), ()),
                ("END".to_string(), ()),
                ("RESW".to_string(), ()),
                ("RESB".to_string(), ()),
                ("BASE".to_string(), ()),
            ]),
            registers: HashMap::from([
                ("A".to_string(), 0x0),
                ("X".to_string(), 0x1),
                ("L".to_string(), 0x2),
                ("B".to_string(), 0x3),
                ("S".to_string(), 0x4),
                ("T".to_string(), 0x5),
                ("F".to_string(), 0x6),
            ]),
        })
    }

    pub fn symbol(&mut self, symbol: &str) -> (u16, bool) {
        return self.symbol_table.get(symbol, 0x0);
    }

    pub fn translate(
        &mut self,
        address: u16,
        user_code: &str,
        line_number: u32,
        original_line: &str,
    ) -> Result<(Code, u16, Vec<u16>), Box<dyn Error>> {
        let mut tmp = String::from(user_code);
        let comment_offset = tmp.find(".").unwrap_or(tmp.len());
        let user_code: String = tmp.drain(..comment_offset).collect();
        let user_code = user_code.trim();

        // return value
        let mut need_modify_obj_code: Vec<u16>;
        need_modify_obj_code = Vec::new();
        let mut move_address: u16 = 3;
        let code = Code::new(user_code, &tmp, line_number);

        if user_code.len() == 0 {
            if self.verbose {
                print!("{}:\t{}\n-> ", line_number, original_line);
                println!("{}", "Empty line");
            }

            move_address = 0;
            code.nocode.set(true);
            return Ok((code, move_address, Vec::new()));
        }

        if self.verbose {
            print!("{}:\t{}\n-> ", line_number, original_line);
        }

        let user_code: Vec<&str> = self.separator.split(&user_code).into_iter().collect();

        if user_code.len() == 1 {
            let mnemonic = user_code[0].trim();
            let byte_code: u8;
            let opcode_format: u8;

            if !self.program_start {
                return Err("code need to start with a legal START mnemonic".into());
            }

            match self.opcode_table.get(mnemonic) {
                Some(instruction) => {
                    (byte_code, opcode_format) = instruction;
                }
                None => {
                    return Err(format!("unknown instruction：{}", mnemonic).into());
                }
            }

            match self.code_translate(address, byte_code, mnemonic, "", opcode_format, 1) {
                Ok((obj_code, byte, _, _, _, _)) => {
                    code.obj_code.set(obj_code);
                    code.byte.set(byte as u16);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        if user_code.len() == 2 {
            let field_1 = user_code[0].trim();
            let field_2 = user_code[1].trim();
            let byte_code: u8;
            let opcode_format: u8;

            if !self.program_start {
                return Err("code need to start with a legal START mnemonic".into());
            }

            // no operand
            if let Some(instruction) = self.opcode_table.get(field_2) {
                (byte_code, opcode_format) = instruction;

                match self.code_translate(address, byte_code, field_2, "", opcode_format, 1) {
                    Ok((obj_code, byte, _, _, _, _)) => {
                        code.obj_code.set(obj_code);
                        code.byte.set(byte as u16);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }

                self.symbol_legal(field_1)?;

                match self.symbol_table.insert(field_1, address) {
                    Ok(wait_byte_code) => {
                        need_modify_obj_code = wait_byte_code;
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            } else {
                let mnemonic = field_1;
                let operand = field_2;
                let format: u8;

                match mnemonic {
                    "BASE" => {
                        move_address = 0;
                        self.symbol_legal(operand)?;
                        if self.wait_for_base.get() {
                            self.wait_for_base.set(false);
                        } else {
                            return Err("BASE must be after LDB".into());
                        }
                        if self.symbol_table.is_legal(operand) {
                            self.base.replace(String::from(operand));
                            if self.verbose {
                                print!("now base is {}", operand);
                            }
                        } else {
                            return Err(format!("illegal operand：{}", operand).into());
                        }
                        code.nocode.set(true);
                    }
                    "END" => {
                        move_address = 0;
                        self.program_end = true;
                        self.symbol_legal(operand)?;
                        let (operand_byte_code, need_alloc) =
                            self.symbol_table.get(operand, address);
                        if need_alloc {
                            return Err(format!("illegal operand：{}", operand).into());
                        } else {
                            self.program_start_address = operand_byte_code;
                            if self.verbose {
                                print!("program execute at 0x{:04X} ", operand_byte_code);
                            }
                        }
                        code.nocode.set(true);
                    }
                    _ => {
                        if mnemonic.bytes().nth(0).unwrap() == b'+' {
                            format = 4;
                        } else {
                            format = 3;
                        }
                        let mnemonic = mnemonic.replace("+", "");
                        if mnemonic == "LDB" {
                            if self.wait_for_base.get() {
                                return Err(
                                    "You are already load to base without BASE mnemonic".into()
                                );
                            } else {
                                self.wait_for_base.set(true);
                            }
                        }
                        match self.opcode_table.get(&mnemonic) {
                            Some(instruction) => {
                                (byte_code, opcode_format) = instruction;
                                match self.code_translate(
                                    address,
                                    byte_code,
                                    &mnemonic,
                                    operand,
                                    opcode_format,
                                    format,
                                ) {
                                    Ok((obj_code, byte, ni, need_alloc, base, operand)) => {
                                        code.obj_code.set(obj_code);
                                        code.byte.set(byte as u16);
                                        code.base.replace(String::from(base));
                                        code.ni.set(ni);
                                        code.operand.replace(String::from(operand));
                                        if need_alloc {
                                            code.undone.set(true);
                                        }
                                    }
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }
                            }
                            None => {
                                return Err(format!("unknown instruction：{}", mnemonic).into());
                            }
                        }
                    }
                }
            }
        }

        if user_code.len() == 3 {
            let label = user_code[0].trim();
            let mnemonic = user_code[1].trim();
            let operand = user_code[2].trim();
            let format: u8;
            let byte_code: u8;
            let opcode_format: u8;

            match mnemonic {
                "START" => {
                    match u16::from_str_radix(operand, 16) {
                        Ok(address) => {
                            move_address = address;
                        }
                        Err(_) => {
                            return Err(format!(
                                "illegal Start address：{} must be positive Hex",
                                operand
                            )
                            .into());
                        }
                    }
                    self.program_start = true;
                    self.program_name = label.to_string();
                    if self.verbose {
                        print!(
                            "program name: {}, program start at 0x{:04X} ",
                            label, move_address
                        );
                    }
                    code.nocode.set(true);
                    code.byte.set(0);
                }
                "RESB" => {
                    match u16::from_str_radix(operand, 10) {
                        Ok(get) => {
                            move_address = get;
                        }
                        Err(_) => {
                            return Err(format!(
                                "illegal：{} must be positive decimal",
                                operand
                            )
                            .into());
                        }
                    }
                    code.nocode.set(true);
                    code.byte.set(move_address);
                    code.variable.set(true);
                }
                "RESW" => {
                    match u16::from_str_radix(operand, 10) {
                        Ok(get) => {
                            move_address = get;
                        }
                        Err(_) => {
                            return Err(format!(
                                "illegal：{} must be positive decimal",
                                operand
                            )
                            .into());
                        }
                    }
                    move_address = move_address * 3;
                    code.nocode.set(true);
                    code.byte.set(move_address * 3);
                    code.variable.set(true);
                }
                "WORD" => {
                    move_address = 3;
                    let obj_code: u32;
                    let num: i16;

                    match i16::from_str_radix(operand, 10) {
                        Ok(get) => {
                            num = get;
                        }
                        Err(_) => {
                            return Err(format!(
                                "illegal：{} must be decimal",
                                operand
                            )
                            .into());
                        }
                    }

                    if num > 2047 || num < -2048 {
                        return Err(format!("operand {} out of range", operand).into());
                    }
                    let operand_str: String = format!("{:03X}", num);
                    let mut operand = String::new();

                    for i in operand_str.len() - 3..operand_str.len() {
                        operand.push(operand_str.chars().nth(i).unwrap());
                    }
                    obj_code = u32::from_str_radix(&format!("{}", operand), 16)?;

                    code.obj_code.set(obj_code);
                    code.byte.set(3);
                    code.variable.set(true);
                }
                "BYTE" => {
                    let obj_code: u32;
                    if operand.len() <= 3 {
                        return Err(format!("illegal operand：{}", operand).into());
                    }
                    if operand.bytes().nth(0).unwrap() == b'C' {
                        let mut tmp_obj_code = String::new();

                        for i in 2..operand.len() - 1 {
                            let tmp = format!("{:02X}", operand.chars().nth(i).unwrap() as u8);
                            tmp_obj_code.push_str(&tmp);
                        }

                        obj_code = u32::from_str_radix(&tmp_obj_code, 16)?;
                        code.byte.set((operand.len() - 3) as u16);
                    } else if operand.bytes().nth(0).unwrap() == b'X' {
                        let mut tmp_obj_code = String::new();
                        let len = (operand.len() - 3) as u32;

                        if len % 2 != 0 {
                            return Err(
                                format!("illegal operand：{} need whole byte", operand).into()
                            );
                        }

                        for i in 2..operand.len() - 1 {
                            tmp_obj_code.push(operand.chars().nth(i).unwrap());
                        }

                        obj_code = u32::from_str_radix(&tmp_obj_code, 16)?;
                        code.byte.set((len / 2) as u16);
                    } else {
                        return Err(format!("unknown operand：{}", operand).into());
                    }

                    code.obj_code.set(obj_code);
                    code.variable.set(true);
                }
                _ => {
                    if !self.program_start {
                        return Err("code need to start with a legal START mnemonic".into());
                    }

                    if mnemonic.bytes().nth(0).unwrap() == b'+' {
                        format = 4;
                    } else {
                        format = 3;
                    }
                    let mnemonic = mnemonic.replace("+", "");

                    match self.opcode_table.get(&mnemonic) {
                        Some(instruction) => {
                            (byte_code, opcode_format) = instruction;

                            match self.code_translate(
                                address,
                                byte_code,
                                &mnemonic,
                                operand,
                                opcode_format,
                                format,
                            ) {
                                Ok((obj_code, byte, ni, need_alloc, base, operand)) => {
                                    code.obj_code.set(obj_code);
                                    code.byte.set(byte as u16);
                                    code.base.replace(String::from(base));
                                    code.operand.replace(String::from(operand));
                                    code.ni.set(ni);
                                    if need_alloc {
                                        code.undone.set(true);
                                    }
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                        None => {
                            return Err(format!("unknown instruction：{}", mnemonic).into());
                        }
                    }
                }
            }

            self.symbol_legal(label)?;

            match self.symbol_table.insert(label, address) {
                Ok(wait_byte_code) => {
                    need_modify_obj_code = wait_byte_code;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        if move_address != 0 {
            code.address.set(address);
            if self.verbose {
                print!("address: 0x{:04X} ", address);
            }
        }

        if !code.nocode.get() {
            let width = (code.byte.get() * 2) as usize;
            if self.verbose {
                print!(
                    "byte code: 0x{:0width$X} ",
                    code.obj_code.get(),
                    width = width
                );
            }
        }
        if self.verbose {
            println!();
        }
        self.program_length
            .set(self.program_length.get() + move_address);

        Ok((code, move_address, need_modify_obj_code))
    }

    fn symbol_legal(&self, label: &str) -> Result<(), Box<dyn Error>> {
        if self.opcode_table.get(label) != None {
            return Err(format!("{} is a instruction", label).into());
        }

        if self.registers.get(label) != None {
            return Err(format!("{} is a register", label).into());
        }

        if self.reserve.get(label) != None {
            return Err(format!("{} is a reserve word", label).into());
        }
        return Ok(());
    }

    fn code_translate(
        &mut self,
        address: u16,
        byte_code: u8,
        mnemonic: &str,
        operand: &str,
        opcode_format: u8,
        format: u8,
    ) -> Result<(u32, u8, u32, bool, String, String), Box<dyn Error>> {
        // return value
        let obj_code: u32;
        let byte: u8;
        let mut need_alloc = false;
        let finial_operand: String;
        let ni: u32;

        match opcode_format {
            1 => {
                obj_code = 0 << 8 + byte_code as u32;
                byte = opcode_format;
                finial_operand = String::new();
                ni = 0;
            }
            2 => {
                let operand: Vec<&str> = self.comma_separator.split(&operand).into_iter().collect();
                if operand.len() == 1 {
                    if let Some(r1) = self.registers.get(operand[0]) {
                        obj_code = u32::from_str_radix(
                            &format!("{:02X}{:01X}{:01X}", byte_code, r1, 0),
                            16,
                        )?;
                    } else {
                        return Err(format!("invalid register：{}", operand[0]).into());
                    }
                } else if operand.len() == 2 {
                    if let Some(r1) = self.registers.get(operand[0]) {
                        if let Some(r2) = self.registers.get(operand[1]) {
                            obj_code = u32::from_str_radix(
                                &format!("{:02X}{:01X}{:01X}", byte_code, r1, r2),
                                16,
                            )?;
                        } else {
                            return Err(format!("invalid register：{}", operand[1]).into());
                        }
                    } else {
                        return Err(format!("invalid register：{}", operand[0]).into());
                    }
                } else {
                    return Err(format!("instruction：{} format error", mnemonic).into());
                }
                byte = opcode_format;
                finial_operand = String::new();
                ni = 0;
            }
            34 => {
                if mnemonic == "RSUB" {
                    obj_code = (byte_code as u32 + Parser::SIMPLE_ADDRESS) << 2 * 8;
                    byte = 3;
                    need_alloc = false;
                    finial_operand = String::new();
                    ni = 0;
                } else {
                    let original_operand = operand;
                    let operand: String;
                    let mut xbpe: u8 = 0x0;
                    let operand_byte_code: u16;

                    if original_operand.bytes().nth(0).unwrap() == b'@' {
                        ni = Parser::INDIRECT_ADDRESS;
                        operand = original_operand.replace("@", "");
                    } else if original_operand.bytes().nth(0).unwrap() == b'#' {
                        ni = Parser::IMMEDIATE_ADDRESS;
                        operand = original_operand.replace("#", "");
                    } else {
                        ni = Parser::SIMPLE_ADDRESS;
                        operand = String::from(original_operand);
                    }

                    let original_operand = operand.clone();
                    let operand: Vec<&str> = self
                        .comma_separator
                        .split(operand.as_str())
                        .into_iter()
                        .collect();

                    if operand.len() == 2 {
                        if operand[1] == "X" {
                            xbpe += 8;
                        } else {
                            return Err(format!("invalid operand：{}", original_operand).into());
                        }
                    }
                    if operand.len() > 2 {
                        return Err(format!("invalid operand：{}", original_operand).into());
                    }

                    finial_operand = String::from(operand[0]);

                    if ni == Parser::IMMEDIATE_ADDRESS {
                        if let Ok(byte) = u16::from_str_radix(operand[0], 10) {
                            operand_byte_code = byte;
                        } else {
                            self.symbol_legal(operand[0])?;
                            (operand_byte_code, need_alloc) =
                                self.symbol_table.get(operand[0], address);
                        }

                        if format == 3 {
                            let operand = operand_byte_code as i32;
                            obj_code = self.fill_obj_code(byte_code, ni, xbpe, operand, 3)?;
                        } else {
                            let operand = operand_byte_code as i32;
                            obj_code = self.fill_obj_code(byte_code, ni, xbpe, operand, 5)?;
                        }
                    } else {
                        self.symbol_legal(operand[0])?;
                        (operand_byte_code, need_alloc) =
                            self.symbol_table.get(operand[0], address);
                        if format == 4 {
                            xbpe += 1;
                        }

                        if need_alloc {
                            if format == 3 {
                                let operand = operand_byte_code as i32;
                                obj_code = self.fill_obj_code(byte_code, ni, xbpe, operand, 3)?;
                            } else {
                                let operand = operand_byte_code as i32;
                                obj_code = self.fill_obj_code(byte_code, ni, xbpe, operand, 5)?;
                            }
                        } else if format == 3 {
                            let pc: i16 = (address + (opcode_format as u16)) as i16;
                            let operand: i32;
                            (operand, need_alloc) =
                                self.fill_operand(address, pc, operand_byte_code)?;
                            obj_code = self.fill_obj_code(byte_code, ni, xbpe, operand, 3)?;
                        } else {
                            let operand = operand_byte_code as i32;
                            obj_code = self.fill_obj_code(byte_code, ni, xbpe, operand, 5)?;
                        }
                    }
                    byte = format;
                }
            }
            _ => {
                return Err(format!("instruction：{} format error", mnemonic).into());
            }
        }

        if need_alloc && self.verbose {
            print!("obj code not done\n-> ");
        }

        Ok((
            obj_code,
            byte,
            ni,
            need_alloc,
            self.base.borrow().clone(),
            finial_operand,
        ))
    }

    fn fill_obj_code(
        &mut self,
        byte_code: u8,
        ni: u32,
        xbpe: u8,
        operand: i32,
        width: usize,
    ) -> Result<u32, Box<dyn Error>> {
        let byte_code = (byte_code as u32) + ni;
        let operand_str: String = format!("{:0width$X}", operand, width = width);
        let mut operand = String::new();

        for i in operand_str.len() - 3..operand_str.len() {
            operand.push(operand_str.chars().nth(i).unwrap());
        }

        let obj_code =
            u32::from_str_radix(&format!("{:02X}{:01X}{}", byte_code, xbpe, operand), 16)?;

        return Ok(obj_code);
    }

    pub fn fill_operand_base(
        &mut self,
        address: u16,
        pc: i16,
        base: &str,
        operand: u16,
    ) -> Result<(i32, bool, u8), Box<dyn Error>> {
        // FIXME:check overflow
        let operand_byte_code = (operand as i16) - pc;

        if operand_byte_code < -2048 || operand_byte_code > 2047 {
            if base == "" {
                return Err(format!("re-alloc: need base, but don't have").into());
            }
            let (base_address, need_alloc) = self.symbol_table.get(base, address);
            let base_address = base_address as i32;
            if need_alloc {
                return Ok((base_address, true, 0));
            } else {
                let disp = (operand as i32) - base_address;
                if disp > 4095 || disp < 0 {
                    return Err(format!(
                        "address 0x{:04X} -> use base {} also out of range",
                        address, base
                    )
                    .into());
                }
                return Ok((disp, need_alloc, 4));
            }
        } else {
            return Ok((operand_byte_code as i32, false, 2));
        }
    }

    fn fill_operand(
        &mut self,
        address: u16,
        pc: i16,
        operand: u16,
    ) -> Result<(i32, bool), Box<dyn Error>> {
        // FIXME:check overflow
        let operand_byte_code = (operand as i16) - pc;

        if operand_byte_code < -2048 || operand_byte_code > 2047 {
            if self.base.borrow().to_string() == "" {
                return Err(format!("need base, but don't have").into());
            }
            let (base_address, need_alloc) = self.symbol_table.get(&self.base.borrow(), address);
            let base_address = base_address as i32;
            if need_alloc {
                return Ok((base_address, true));
            } else {
                let base = (operand as i32) - base_address;
                if base > 4095 || base < 0 {
                    return Err(format!("use base {} also out of range", self.base.borrow()).into());
                }
                return Ok((base, need_alloc));
            }
        } else {
            return Ok((operand_byte_code as i32, false));
        }
    }

    pub fn symbols_inter(&self) {
        self.symbol_table.inter();
    }
}
