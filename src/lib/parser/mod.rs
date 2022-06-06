use regex::Regex;
use std::collections::HashMap;

mod opcode_table;
use opcode_table::OpcodeTable;
mod symbol_table;
use symbol_table::SymbolTable;

#[path = "../err.rs"]
mod err;
#[path = "../log/mod.rs"]
mod log;

pub struct Code {
    pub obj_code: u64,
    _char_obj_code: String,
    _char_mode: bool,
    pub location: u32,
    pub byte: u32,
    pub ni: u8,
    pub variable: bool,
    pub no_obj_code: bool,
    pub source_code: String,
    pub line_number: u32,
    pub undone: bool,
    pub base: String,
    pub operand: String,
}

impl Code {
    pub fn new(
        line_number: u32,
        source_code: &str,
        location: u32,
        ni: u8,
        base: String,
        operand: String,
        obj_code: u64,
        byte: u32,
        variable: bool,
        no_obj_code: bool,
        undone: bool,
    ) -> Code {
        Code {
            line_number: line_number,
            source_code: String::from(source_code),
            location: location,
            ni: ni,
            base: base,
            operand: String::from(operand),
            obj_code: obj_code,
            byte: byte,
            variable: variable,
            no_obj_code: no_obj_code,
            undone: undone,
            _char_obj_code: String::new(),
            _char_mode: false,
        }
    }

    pub fn empty(line_number: u32, source_code: String) -> Code {
        Code {
            line_number: line_number,
            source_code: source_code,
            ni: 0,
            base: String::new(),
            operand: String::new(),
            obj_code: 0,
            location: 0,
            byte: 0,
            variable: false,
            no_obj_code: true,
            undone: false,
            _char_obj_code: String::new(),
            _char_mode: false,
        }
    }

    pub fn re_alloc(&mut self, parser: &mut Parser) -> Result<(), String> {
        let pc: i32;
        let operand_location: u32;

        match self.location.checked_add(self.byte as u32) {
            Some(new_length) => {
                pc = new_length as i32;
            }
            None => {
                return Err(err::handler().e306());
            }
        }

        match parser.get_symbol_location(&self.operand) {
            Some((location, need_alloc)) => {
                if need_alloc {
                    return Err(err::handler().e999(&format!(
                        "operand {} is not done, but call re_alloc",
                        self.operand
                    )));
                }
                operand_location = location;
            }
            None => {
                return Err(err::handler().e999(&format!(
                    "operand {} is not defined, but call re_alloc",
                    self.operand
                )));
            }
        }

        if self.xbpe() % 2 == 1 {
            self.obj_code += operand_location as u64;
            self.undone = false;
            return Ok(());
        }

        let (operand_obj_code, need_alloc, move_xbpe) =
            parser.fill_operand(self.location, pc, self.base.as_ref(), operand_location)?;
        if need_alloc {
            return Ok(());
        }

        let xbpe = self.xbpe() + move_xbpe;
        let operand_str: String =
            format!("{:0width$X}", operand_obj_code, width = self.byte as usize);
        let mut operand = String::new();

        for i in operand_str.len() - (self.byte as usize)..operand_str.len() {
            operand.push(operand_str.chars().nth(i).unwrap());
        }
        let operand_obj_code: u64;
        if self.byte == 4 {
            operand_obj_code = u64::from_str_radix(&operand, 16).unwrap() + ((xbpe as u64) << 20);
        } else {
            operand_obj_code = u64::from_str_radix(&operand, 16).unwrap() + ((xbpe as u64) << 12);
        }
        self.obj_code += operand_obj_code;
        self.undone = false;

        return Ok(());
    }

    fn xbpe(&self) -> u8 {
        if self.byte <= 2 {
            return 0;
        }
        let xbpe_bit = format!("{:0width$X}", self.obj_code, width = self.byte as usize)
            .chars()
            .nth(2)
            .unwrap();
        return u8::from_str_radix(xbpe_bit.to_string().as_str(), 16).unwrap();
    }
}

enum AddressingMode {
    Immediate = 0b01,
    Indirect = 0b10,
    Simple = 0b11,
}

pub enum SizeLimit {
    Location = 0xfffff, //2^20
}

pub struct Parser {
    opcode_table: OpcodeTable,
    symbol_table: SymbolTable,
    pub program_name: String,
    program_start: bool,
    pub program_start_address: u32,
    pub program_end: bool,
    pub program_length: u32,
    base: String,
    wait_for_base: bool,
    space_separator: Regex,
    char_separator: Regex,
    comma_separator: Regex,
    reserve: HashMap<String, ()>,
    registers: HashMap<String, u8>,
    verbose: bool,
}

impl Parser {
    pub fn new(verbose: bool) -> Parser {
        Parser {
            opcode_table: OpcodeTable::new(),
            symbol_table: SymbolTable::new(),
            program_name: String::new(),
            program_start: false,
            program_start_address: 0x0,
            program_end: false,
            program_length: 0x0,
            base: String::new(),
            wait_for_base: false,
            space_separator: Regex::new(r"[ \t]+").unwrap(),
            char_separator: Regex::new(r"C'").unwrap(),
            comma_separator: Regex::new(r",").unwrap(),
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
            verbose: verbose,
        }
    }

    pub fn get_symbol_location(&self, symbol: &str) -> Option<(u32, bool)> {
        return self.symbol_table.get_location(symbol);
    }

    pub fn symbols_inter(&self) {
        self.symbol_table.inter();
    }

    pub fn translate(
        &mut self,
        line_number: u32,
        location: u32,
        user_code: &str,
        source_code: &str,
    ) -> Result<(Code, u32, Vec<u32>), String> {
        // return Value
        let (code, offset, need_modify_code): (Code, u32, Vec<u32>);

        // remove comment
        let mut user_code = String::from(user_code);
        let comment_offset = user_code.find(".").unwrap_or(user_code.len());
        let user_code: String = user_code.drain(..comment_offset).collect();
        let user_code = user_code.trim();

        if user_code.len() == 0 {
            log::println(
                &format!("{}:\t{}\n-> {}", line_number, source_code, "Empty line"),
                self.verbose,
            );
            code = Code::empty(line_number, String::from(source_code));
            offset = 0;
            need_modify_code = vec![];
            return Ok((code, offset, need_modify_code));
        }

        log::print(
            &format!("{}:\t{}\n-> ", line_number, source_code),
            self.verbose,
        );

        // separate source code by space and tab
        let mut result: Vec<&str>;
        match self.char_separator.find(&user_code) {
            Some(mat) => {
                let code_1 = &user_code[..mat.start()];
                let code_1 = code_1.trim();
                let code_2 = &user_code[mat.start()..];
                result = self.space_separator.split(code_1).collect::<Vec<&str>>();
                result.push(code_2);
            }
            None => {
                result = self
                    .space_separator
                    .split(&user_code)
                    .collect::<Vec<&str>>();
            }
        }
        let user_code = result.as_slice();

        match user_code.len() {
            1 => {
                let mnemonic = user_code[0].trim();
                let (opcode, instruction_format): (u8, u8);

                if !self.program_start {
                    return Err(err::handler().e301());
                }

                if self.wait_for_base && mnemonic != "BASE" {
                    return Err(err::handler().e303());
                }

                match self.opcode_table.get(mnemonic) {
                    Some(instruction) => {
                        (opcode, instruction_format) = *instruction;
                    }
                    None => {
                        return Err(err::handler().e201(mnemonic));
                    }
                }

                match self.code_translate(location, mnemonic, opcode, "", instruction_format, false)
                {
                    Ok((ni, obj_code, finial_operand, undone, byte)) => {
                        offset = byte as u32;
                        code = Code::new(
                            line_number,
                            source_code,
                            location,
                            ni,
                            self.base.clone(),
                            finial_operand,
                            obj_code,
                            offset,
                            false,
                            false,
                            undone,
                        );
                        need_modify_code = vec![];
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            2 => {
                let (field_1, field_2) = (user_code[0].trim(), user_code[1].trim());
                let (opcode, instruction_format): (u8, u8);

                if !self.program_start {
                    return Err(err::handler().e301());
                }

                // Case 1: no operand
                if let Some(instruction) = self.opcode_table.get(field_2) {
                    (opcode, instruction_format) = *instruction;
                    let (label, mnemonic) = (field_1, field_2);

                    if self.wait_for_base && mnemonic != "BASE" {
                        return Err(err::handler().e303());
                    }

                    match self.code_translate(
                        location,
                        mnemonic,
                        opcode,
                        "",
                        instruction_format,
                        false,
                    ) {
                        Ok((ni, obj_code, finial_operand, undone, byte)) => {
                            offset = byte as u32;
                            code = Code::new(
                                line_number,
                                source_code,
                                location,
                                ni,
                                self.base.clone(),
                                finial_operand,
                                obj_code,
                                offset,
                                false,
                                false,
                                undone,
                            );
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }

                    if let Err(e) = self.symbol_legal(label) {
                        return Err(format!("label invalid: {}", e));
                    }

                    match self.symbol_table.insert(label, location) {
                        Ok(waiting_list) => {
                            need_modify_code = waiting_list;
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                } else {
                    // Case 2: have operand
                    let (mnemonic, operand) = (field_1, field_2);

                    if self.wait_for_base && mnemonic != "BASE" {
                        return Err(err::handler().e303());
                    }

                    match mnemonic {
                        "BASE" => {
                            offset = 0;

                            if let Err(e) = self.symbol_legal(operand) {
                                return Err(format!("operand invalid: {}", e));
                            }
                            if self.wait_for_base {
                                self.wait_for_base = false;
                            } else {
                                return Err(err::handler().e302());
                            }

                            self.base = String::from(operand);
                            log::print(&format!("now base is {}", operand), self.verbose);

                            code = Code::empty(line_number, String::from(source_code));
                            need_modify_code = vec![];
                        }
                        "END" => {
                            let (operand_location, need_alloc): (u32, bool);
                            offset = 0;

                            if self.program_end {
                                return Err(err::handler().e303());
                            }
                            self.program_end = true;

                            if let Err(e) = self.symbol_legal(operand) {
                                return Err(format!("operand invalid: {}", e));
                            }

                            match self.symbol_table.get_location(operand) {
                                Some(res) => {
                                    (operand_location, need_alloc) = (res.0, res.1);
                                }
                                None => {
                                    return Err(err::handler().e202());
                                }
                            }

                            if need_alloc {
                                return Err(err::handler().e202());
                            }
                            code = Code::empty(line_number, String::from(source_code));
                            need_modify_code = vec![];
                            self.program_start_address = operand_location;
                            log::print(
                                &format!("program execute at 0x{:04X} ", operand_location),
                                self.verbose,
                            );
                        }
                        _ => {
                            let original_mnemonic = mnemonic.clone();
                            let (extension, mnemonic): (bool, &str);

                            if original_mnemonic.bytes().nth(0).unwrap() == b'+' {
                                extension = true;
                                mnemonic = &original_mnemonic[1..];
                            } else {
                                extension = false;
                                mnemonic = &original_mnemonic[..];
                            }

                            if mnemonic == "LDB" {
                                self.wait_for_base = true;
                            }

                            match self.opcode_table.get(mnemonic) {
                                Some(instruction) => {
                                    (opcode, instruction_format) = *instruction;
                                }
                                None => {
                                    return Err(err::handler().e201(mnemonic));
                                }
                            }
                            match self.code_translate(
                                location,
                                mnemonic,
                                opcode,
                                operand,
                                instruction_format,
                                extension,
                            ) {
                                Ok((ni, obj_code, finial_operand, undone, byte)) => {
                                    offset = byte as u32;
                                    code = Code::new(
                                        line_number,
                                        source_code,
                                        location,
                                        ni,
                                        self.base.clone(),
                                        finial_operand,
                                        obj_code,
                                        offset,
                                        false,
                                        false,
                                        undone,
                                    );
                                    need_modify_code = vec![];
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }
                        }
                    }
                }
            }
            3 => {
                let label = user_code[0].trim();
                let mnemonic = user_code[1].trim();
                let operand = user_code[2].trim();
                let (opcode, instruction_format): (u8, u8);

                if self.wait_for_base && mnemonic != "BASE" {
                    return Err(err::handler().e303());
                }

                match mnemonic {
                    "START" => {
                        match u32::from_str_radix(operand, 16) {
                            Ok(location) => {
                                offset = location;
                            }
                            Err(_) => {
                                return Err(err::handler().e203(operand));
                            }
                        }
                        self.program_start = true;
                        if label.len() > 6 {
                            return Err(err::handler().e305(label));
                        }
                        self.program_name = String::from(label);
                        log::print(
                            &format!(
                                "program name: {}, program start at 0x{:04X} ",
                                label, offset
                            ),
                            self.verbose,
                        );
                        code = Code::empty(line_number, String::from(source_code));
                    }
                    "RESB" => {
                        match u32::from_str_radix(operand, 10) {
                            Ok(get) => {
                                offset = get;
                            }
                            Err(_) => {
                                return Err(err::handler().e204());
                            }
                        }
                        if offset >= SizeLimit::Location as u32 {
                            return Err(err::handler().e204());
                        }
                        code = Code::new(
                            line_number,
                            source_code,
                            location,
                            0,
                            self.base.clone(),
                            String::from(operand),
                            0,
                            offset,
                            true,
                            true,
                            false,
                        );
                    }
                    "RESW" => {
                        let size: u32;
                        match u32::from_str_radix(operand, 10) {
                            Ok(get) => {
                                size = get;
                            }
                            Err(_) => {
                                return Err(err::handler().e204());
                            }
                        }
                        match size.checked_mul(3) {
                            Some(get) => {
                                offset = get;
                            }
                            None => {
                                return Err(err::handler().e204());
                            }
                        }
                        if offset >= SizeLimit::Location as u32 {
                            return Err(err::handler().e204());
                        }
                        code = Code::new(
                            line_number,
                            source_code,
                            location,
                            0,
                            self.base.clone(),
                            String::from(operand),
                            0,
                            offset,
                            true,
                            true,
                            false,
                        );
                    }
                    "WORD" => {
                        offset = 3;
                        let num: i16;

                        match i16::from_str_radix(operand, 10) {
                            Ok(get) => {
                                num = get;
                            }
                            Err(_) => {
                                return Err(err::handler().e205());
                            }
                        }
                        if num > 2047 || num < -2048 {
                            return Err(err::handler().e205());
                        }

                        let operand_str: String = format!("{:03X}", num);
                        let mut operand = String::new();

                        for i in operand_str.len() - 3..operand_str.len() {
                            operand.push(operand_str.chars().nth(i).unwrap());
                        }
                        let obj_code = u64::from_str_radix(&format!("{}", operand), 16).unwrap();

                        code = Code::new(
                            line_number,
                            source_code,
                            location,
                            0,
                            self.base.clone(),
                            String::from(operand),
                            obj_code,
                            offset,
                            true,
                            false,
                            false,
                        );
                    }
                    "BYTE" => {
                        let mut tmp_obj_code = String::new();

                        if operand.len() <= 3 {
                            return Err(err::handler().e206());
                        }
                        let quote_s = operand.bytes().nth(1).unwrap();
                        let quote_e = operand.bytes().nth(operand.len() - 1).unwrap();
                        if quote_s != b'\'' || quote_e != b'\'' {
                            return Err(err::handler().e206());
                        }

                        match operand.bytes().nth(0).unwrap() {
                            b'C' => {
                                for i in 2..operand.len() - 1 {
                                    let letter = operand.chars().nth(i).unwrap();

                                    if !letter.is_ascii() {
                                        return Err(err::handler().e207());
                                    }

                                    tmp_obj_code.push_str(&format!("{:02X}", letter as u8));
                                }
                            }
                            b'X' => {
                                if (operand.len() - 3) % 2 != 0 {
                                    return Err(err::handler().e209());
                                }
                                for i in 2..operand.len() - 1 {
                                    tmp_obj_code.push(operand.chars().nth(i).unwrap());
                                }
                            }
                            _ => {
                                return Err(err::handler().e206());
                            }
                        }

                        if tmp_obj_code.len() > 16 {
                            return Err(err::handler().e208());
                        }

                        offset = (tmp_obj_code.len() / 2) as u32;
                        code = Code::new(
                            line_number,
                            source_code,
                            location,
                            0,
                            self.base.clone(),
                            String::from(operand),
                            u64::from_str_radix(&tmp_obj_code, 16).unwrap(),
                            offset,
                            true,
                            false,
                            false,
                        );
                    }
                    _ => {
                        if !self.program_start {
                            return Err(err::handler().e301());
                        }

                        let original_mnemonic = mnemonic.clone();
                        let (extension, mnemonic): (bool, &str);

                        if original_mnemonic.bytes().nth(0).unwrap() == b'+' {
                            extension = true;
                            mnemonic = &original_mnemonic[1..];
                        } else {
                            extension = false;
                            mnemonic = &original_mnemonic[..];
                        }

                        if mnemonic == "LDB" {
                            self.wait_for_base = true;
                        }

                        match self.opcode_table.get(mnemonic) {
                            Some(instruction) => {
                                (opcode, instruction_format) = *instruction;
                            }
                            None => {
                                return Err(err::handler().e201(mnemonic));
                            }
                        }
                        match self.code_translate(
                            location,
                            mnemonic,
                            opcode,
                            operand,
                            instruction_format,
                            extension,
                        ) {
                            Ok((ni, obj_code, finial_operand, undone, byte)) => {
                                offset = byte as u32;
                                if finial_operand == label {
                                    self.symbol_table.remove_waiting(&finial_operand, location);
                                    return Err(err::handler().e313());
                                }
                                code = Code::new(
                                    line_number,
                                    source_code,
                                    location,
                                    ni,
                                    self.base.clone(),
                                    finial_operand,
                                    obj_code,
                                    offset,
                                    false,
                                    false,
                                    undone,
                                );
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                }
                if let Err(e) = self.symbol_legal(label) {
                    self.symbol_table.remove_waiting(&code.operand, location);
                    return Err(format!("label invalid: {}", e));
                }
                if mnemonic == "START" {
                    match self.symbol_table.insert(label, offset) {
                        Ok(waiting_list) => {
                            need_modify_code = waiting_list;
                        }
                        Err(e) => {
                            self.symbol_table.remove_waiting(&code.operand, offset);
                            return Err(e);
                        }
                    }
                } else {
                    match self.symbol_table.insert(label, location) {
                        Ok(waiting_list) => {
                            need_modify_code = waiting_list;
                        }
                        Err(e) => {
                            self.symbol_table.remove_waiting(&code.operand, location);
                            return Err(e);
                        }
                    }
                }
            }
            _ => {
                return Err(err::handler().e310());
            }
        }

        if !code.no_obj_code {
            let width = (code.byte * 2) as usize;
            log::print(
                &format!("byte code: 0x{:0width$X} ", code.obj_code, width = width),
                self.verbose,
            );
        }
        log::println("", self.verbose);

        match self.program_length.checked_add(offset) {
            Some(new_length) => {
                self.program_length = new_length;
            }
            None => {
                return Err(err::handler().e306());
            }
        }
        if self.program_length > SizeLimit::Location as u32 {
            return Err(err::handler().e306());
        }

        Ok((code, offset, need_modify_code))
    }

    pub fn fill_operand(
        &mut self,
        location: u32,
        pc: i32,
        base: &str,
        operand_location: u32,
    ) -> Result<(i32, bool, u8), String> {
        let operand_location = operand_location as i32;
        let operand_obj_code = operand_location - pc;

        if operand_obj_code < -2048 || operand_obj_code > 2047 {
            if base == "" {
                return Err(err::handler().e307());
            }
            let (base_address, need_alloc) =
                self.symbol_table.get_location_or_create(base, location)?;
            let base_address = base_address as i32;
            if need_alloc {
                return Ok((0, true, 0));
            }

            let disp = operand_location - base_address;
            if disp > 4095 || disp < 0 {
                return Err(err::handler().e308(base));
            }

            return Ok((disp, need_alloc, 4));
        }
        return Ok((operand_obj_code, false, 2));
    }

    fn code_translate(
        &mut self,
        location: u32,
        mnemonic: &str,
        opcode: u8,
        original_operand: &str,
        instruction_format: u8,
        extension: bool,
    ) -> Result<(u8, u64, String, bool, u8), String> {
        let (ni, obj_code, finial_operand, undone, byte): (u8, u64, String, bool, u8);

        match instruction_format {
            1 => {
                if !original_operand.eq("") {
                    return Err(err::handler().e210(mnemonic));
                }

                ni = 0;
                obj_code = 0 << 8 + opcode as u32;
                finial_operand = String::new();
                undone = false;
                byte = instruction_format;
            }
            2 => {
                if original_operand.eq("") {
                    return Err(err::handler().e211(mnemonic));
                }
                ni = 0;
                finial_operand = String::from(original_operand);
                undone = false;
                byte = instruction_format;

                let operand: Vec<&str> = self.comma_separator.split(&original_operand).collect();

                match operand.len() {
                    1 => {
                        if let Some(r1) = self.registers.get(operand[0]) {
                            obj_code = u64::from_str_radix(
                                &format!("{:02X}{:01X}{:01X}", opcode, r1, 0),
                                16,
                            )
                            .unwrap();
                        } else {
                            return Err(err::handler().e212(operand[0]));
                        }
                    }
                    2 => {
                        if operand[0] == "" || operand[1] == "" {
                            return Err(err::handler().e213());
                        }
                        if let Some(r1) = self.registers.get(operand[0]) {
                            if let Some(r2) = self.registers.get(operand[1]) {
                                obj_code = u64::from_str_radix(
                                    &format!("{:02X}{:01X}{:01X}", opcode, r1, r2),
                                    16,
                                )
                                .unwrap();
                            } else {
                                return Err(err::handler().e212(operand[1]));
                            }
                        } else {
                            return Err(err::handler().e212(operand[0]));
                        }
                    }
                    _ => {
                        return Err(err::handler().e213());
                    }
                }
            }
            34 => {
                if mnemonic == "RSUB" {
                    if original_operand != "" {
                        return Err(err::handler().e210(mnemonic));
                    }

                    ni = AddressingMode::Simple as u8;
                    obj_code = (opcode as u64 + AddressingMode::Simple as u64) << 2 * 8;
                    finial_operand = String::from(original_operand);
                    undone = false;
                    byte = 3;
                } else {
                    let operand: String;
                    let mut xbpe: u8 = 0x0;
                    let operand_location: u32;

                    if original_operand.bytes().nth(0).unwrap() == b'@' {
                        ni = AddressingMode::Indirect as u8;
                        operand = original_operand.replace("@", "");
                    } else if original_operand.bytes().nth(0).unwrap() == b'#' {
                        ni = AddressingMode::Immediate as u8;
                        operand = original_operand.replace("#", "");
                    } else {
                        ni = AddressingMode::Simple as u8;
                        operand = String::from(original_operand);
                    }
                    let operand: Vec<&str> = self.comma_separator.split(operand.as_str()).collect();

                    if operand.len() == 2 {
                        if operand[1] == "X" {
                            xbpe += 8;
                        } else {
                            return Err(err::handler().e214());
                        }
                    }
                    if operand.len() > 2 {
                        return Err(err::handler().e214());
                    }

                    finial_operand = String::from(operand[0]);

                    let mut num: i32 = -1;
                    let mut is_digit = false;
                    if ni == AddressingMode::Immediate as u8 {
                        if let Ok(n) = i32::from_str_radix(operand[0], 10) {
                            num = n;
                            is_digit = true;
                        }
                    }

                    if ni == AddressingMode::Immediate as u8 && is_digit {
                        let operand = num;
                        xbpe += 1;
                        byte = 4;
                        obj_code = self.fill_obj_code(opcode, ni, xbpe, operand, true);
                        undone = false;
                    } else {
                        if extension {
                            xbpe += 1;
                            byte = 4;
                        } else {
                            byte = 3;
                        }

                        if let Err(e) = self.symbol_legal(operand[0]) {
                            return Err(format!("operand invalid: {}", e));
                        }

                        let mut need_alloc: bool;
                        (operand_location, need_alloc) = self
                            .symbol_table
                            .get_location_or_create(operand[0], location)?;

                        if need_alloc || extension {
                            let operand_obj_code = operand_location as i32;
                            obj_code =
                                self.fill_obj_code(opcode, ni, xbpe, operand_obj_code, extension);
                        } else {
                            let pc: i32;
                            let (operand_obj_code, move_xbpe): (i32, u8);
                            match location.checked_add(byte as u32) {
                                Some(new_length) => {
                                    pc = new_length as i32;
                                }
                                None => {
                                    self.symbol_table.remove_waiting(operand[0], location);
                                    return Err(err::handler().e306());
                                }
                            }

                            (operand_obj_code, need_alloc, move_xbpe) = self.fill_operand(
                                location,
                                pc,
                                self.base.clone().as_ref(),
                                operand_location,
                            )?;
                            obj_code = self.fill_obj_code(
                                opcode,
                                ni,
                                xbpe + move_xbpe,
                                operand_obj_code,
                                extension,
                            );
                        }

                        undone = need_alloc;
                    }
                }
            }
            _ => {
                return Err(err::handler().e999(&format!(
                    "instruction_format error: found {} is {}",
                    mnemonic, instruction_format
                )));
            }
        }

        Ok((ni, obj_code, finial_operand, undone, byte))
    }

    fn symbol_legal(&self, label: &str) -> Result<(), String> {
        if !self.symbol_table.is_legal(label) {
            return Err(err::handler().e101(label));
        }

        if self.opcode_table.contains_key(label) {
            return Err(err::handler().e103(label));
        }

        if self.registers.contains_key(label) {
            return Err(err::handler().e104(label));
        }

        if self.reserve.contains_key(label) {
            return Err(err::handler().e105(label));
        }

        return Ok(());
    }

    fn fill_obj_code(&self, opcode: u8, ni: u8, xbpe: u8, operand: i32, extension: bool) -> u64 {
        let width: usize;

        if extension {
            width = 5
        } else {
            width = 3
        }

        let byte_code = opcode + ni;
        let operand_str: String = format!("{:0width$X}", operand, width = width);
        let mut operand = String::new();

        for i in operand_str.len() - width..operand_str.len() {
            operand.push(operand_str.chars().nth(i).unwrap());
        }

        return u64::from_str_radix(&format!("{:02X}{:01X}{}", byte_code, xbpe, operand), 16)
            .unwrap();
    }
}
