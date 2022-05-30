use std::collections::HashMap;
use std::error::Error;

pub struct OpcodeTable {
    opcodes: HashMap<String, (u8, u8)>,
}

impl OpcodeTable {
    pub fn new() -> Result<OpcodeTable, Box<dyn Error>> {
        // cspell:disable
        let opcodes = HashMap::from([
            ("ADD".to_string(),     (u8::from_str_radix("18", 16)?, 34)),
            ("ADDF".to_string(),    (u8::from_str_radix("58", 16)?, 34)),
            ("ADDR".to_string(),    (u8::from_str_radix("90", 16)?, 2)),
            ("AND".to_string(),     (u8::from_str_radix("40", 16)?, 34)),
            ("CLEAR".to_string(),   (u8::from_str_radix("B4", 16)?, 2)),
            ("COMP".to_string(),    (u8::from_str_radix("28", 16)?, 34)),
            ("COMPF".to_string(),   (u8::from_str_radix("88", 16)?, 34)),
            ("COMPR".to_string(),   (u8::from_str_radix("A0", 16)?, 2)),
            ("DIV".to_string(),     (u8::from_str_radix("24", 16)?, 34)),
            ("DIVF".to_string(),    (u8::from_str_radix("64", 16)?, 34)),
            ("DIVR".to_string(),    (u8::from_str_radix("9C", 16)?, 2)),
            ("FIX".to_string(),     (u8::from_str_radix("C4", 16)?, 1)),
            ("FLOAT".to_string(),   (u8::from_str_radix("C0", 16)?, 1)),
            ("HIO".to_string(),     (u8::from_str_radix("F4", 16)?, 1)),
            ("J".to_string(),       (u8::from_str_radix("3C", 16)?, 34)),
            ("JEQ".to_string(),     (u8::from_str_radix("30", 16)?, 34)),
            ("JGT".to_string(),     (u8::from_str_radix("34", 16)?, 34)),
            ("JLT".to_string(),     (u8::from_str_radix("38", 16)?, 34)),
            ("JSUB".to_string(),    (u8::from_str_radix("48", 16)?, 34)),
            ("LDA".to_string(),     (u8::from_str_radix("00", 16)?, 34)),
            ("LDB".to_string(),     (u8::from_str_radix("68", 16)?, 34)),
            ("LDCH".to_string(),    (u8::from_str_radix("50", 16)?, 34)),
            ("LDF".to_string(),     (u8::from_str_radix("70", 16)?, 34)),
            ("LDL".to_string(),     (u8::from_str_radix("08", 16)?, 34)),
            ("LDS".to_string(),     (u8::from_str_radix("6C", 16)?, 34)),
            ("LDT".to_string(),     (u8::from_str_radix("74", 16)?, 34)),
            ("LDX".to_string(),     (u8::from_str_radix("04", 16)?, 34)),
            ("LPS".to_string(),     (u8::from_str_radix("D0", 16)?, 34)),
            ("MUL".to_string(),     (u8::from_str_radix("20", 16)?, 34)),
            ("MULF".to_string(),    (u8::from_str_radix("60", 16)?, 34)),
            ("MULR".to_string(),    (u8::from_str_radix("98", 16)?, 2)),
            ("NORM".to_string(),    (u8::from_str_radix("C8", 16)?, 1)),
            ("OR".to_string(),      (u8::from_str_radix("44", 16)?, 34)),
            ("RD".to_string(),      (u8::from_str_radix("D8", 16)?, 34)),
            ("RMO".to_string(),     (u8::from_str_radix("AC", 16)?, 2)),
            ("RSUB".to_string(),    (u8::from_str_radix("4C", 16)?, 34)),
            ("SHIFTL".to_string(),  (u8::from_str_radix("A4", 16)?, 2)),
            ("SHIFTR".to_string(),  (u8::from_str_radix("A8", 16)?, 2)),
            ("SIO".to_string(),     (u8::from_str_radix("F0", 16)?, 1)),
            ("SSK".to_string(),     (u8::from_str_radix("EC", 16)?, 34)),
            ("STA".to_string(),     (u8::from_str_radix("0C", 16)?, 34)),
            ("STB".to_string(),     (u8::from_str_radix("78", 16)?, 34)),
            ("STCH".to_string(),    (u8::from_str_radix("54", 16)?, 34)),
            ("STF".to_string(),     (u8::from_str_radix("80", 16)?, 34)),
            ("STI".to_string(),     (u8::from_str_radix("D4", 16)?, 34)),
            ("STL".to_string(),     (u8::from_str_radix("14", 16)?, 34)),
            ("STS".to_string(),     (u8::from_str_radix("7C", 16)?, 34)),
            ("STSW".to_string(),    (u8::from_str_radix("E8", 16)?, 34)),
            ("STT".to_string(),     (u8::from_str_radix("84", 16)?, 34)),
            ("STX".to_string(),     (u8::from_str_radix("10", 16)?, 34)),
            ("SUB".to_string(),     (u8::from_str_radix("1C", 16)?, 34)),
            ("SUBF".to_string(),    (u8::from_str_radix("5C", 16)?, 34)),
            ("SUBR".to_string(),    (u8::from_str_radix("94", 16)?, 2)),
            ("SVC".to_string(),     (u8::from_str_radix("B0", 16)?, 2)),
            ("TD".to_string(),      (u8::from_str_radix("E0", 16)?, 34)),
            ("TIO".to_string(),     (u8::from_str_radix("F8", 16)?, 1)),
            ("TIX".to_string(),     (u8::from_str_radix("2C", 16)?, 34)),
            ("TIXR".to_string(),    (u8::from_str_radix("B8", 16)?, 2)),
            ("WD".to_string(),      (u8::from_str_radix("DC", 16)?, 34)),
        ]);
        // cspell:enable

        Ok(OpcodeTable { opcodes })
    }

    pub fn get(&self, mnemonic: &str) -> Option<(u8, u8)> {
        self.opcodes.get(mnemonic).map(|&opcode| opcode)
    }
}
