use std::env;

pub struct EN;
pub struct ZH;

pub trait ErrMsg {
    fn e001(&self) -> String;
    fn e002(&self) -> String;
    fn e003(&self, msg: &str) -> String;
    fn e101(&self, symbol: &str) -> String;
    fn e102(&self, symbol: &str) -> String;
    fn e103(&self, symbol: &str) -> String;
    fn e104(&self, symbol: &str) -> String;
    fn e105(&self, symbol: &str) -> String;
    fn e201(&self, mnemonic: &str) -> String;
    fn e202(&self) -> String;
    fn e203(&self, location: &str) -> String;
    fn e204(&self) -> String;
    fn e205(&self) -> String;
    fn e206(&self) -> String;
    fn e207(&self) -> String;
    fn e208(&self) -> String;
    fn e209(&self) -> String;
    fn e210(&self, mnemonic: &str) -> String;
    fn e211(&self, mnemonic: &str) -> String;
    fn e212(&self, register: &str) -> String;
    fn e213(&self) -> String;
    fn e214(&self) -> String;
    fn e301(&self) -> String;
    fn e302(&self) -> String;
    fn e303(&self) -> String;
    fn e304(&self) -> String;
    fn e305(&self, program_name: &str) -> String;
    fn e306(&self) -> String;
    fn e307(&self) -> String;
    fn e308(&self, base: &str) -> String;
    fn e309(&self) -> String;
    fn e310(&self) -> String;
    fn e311(&self, operand: &str) -> String;
    fn e312(&self, operand: &str, base: &str) -> String;
    fn e313(&self) -> String;
    fn e999(&self, msg: &str) -> String;
}

impl ErrMsg for EN {
    fn e001(&self) -> String {
        return format!("E[001]: Invalid file name");
    }
    fn e002(&self) -> String {
        return format!("E[002]: Can't open file");
    }
    fn e003(&self, msg: &str) -> String {
        return format!("E[003]: Can not write file: {}", msg);
    }
    fn e101(&self, symbol: &str) -> String {
        return format!("E[101]: Illegal symbol: {}", symbol);
    }
    fn e102(&self, symbol: &str) -> String {
        return format!("E[102]: symbol {} already defined", symbol);
    }
    fn e103(&self, symbol: &str) -> String {
        return format!("E[103]: symbol {} is a mnemonic", symbol);
    }
    fn e104(&self, symbol: &str) -> String {
        return format!("E[104]: symbol {} is a register", symbol);
    }
    fn e105(&self, symbol: &str) -> String {
        return format!("E[105]: symbol {} is a reserved word", symbol);
    }
    fn e201(&self, mnemonic: &str) -> String {
        return format!("E[201]: unknown mnemonic: {}", mnemonic);
    }
    fn e202(&self) -> String {
        return format!("E[202]: end at unknown location");
    }
    fn e203(&self, location: &str) -> String {
        return format!(
            "E[203]: Illegal Start address???{} must be positive 16 bit Hex",
            location
        );
    }
    fn e204(&self) -> String {
        return format!(
            "E[204]: {}, {}",
            "Illegal size: must be positive decimal",
            "and small then 1048576 (word size is 3 bytes)"
        );
    }
    fn e205(&self) -> String {
        return format!("E[205]: Illegal number: must be decimal, and the range is -2048 to 2047");
    }
    fn e206(&self) -> String {
        // cspell:disable
        return format!("E[206]: invalid format: The BYTE format is C'xxxx' or X'xxxx'");
        // cspell:enable
    }
    fn e207(&self) -> String {
        return format!("E[207]: BYTE only support ASCII character");
    }
    fn e208(&self) -> String {
        return format!("E[208]: BYTE only support 8 character");
    }
    fn e209(&self) -> String {
        return format!("E[209]: BYTE X mode need whole hex, EX: F => 0F");
    }
    fn e210(&self, mnemonic: &str) -> String {
        return format!("E[210]: mnemonic {} doesn't have operand", mnemonic);
    }
    fn e211(&self, mnemonic: &str) -> String {
        return format!("E[211]: mnemonic {} have operand", mnemonic);
    }
    fn e212(&self, register: &str) -> String {
        return format!("E[212]: register {} is not exist", register);
    }
    fn e213(&self) -> String {
        return format!("E[213]: Illegal operand format: format is [register] or [register, register]");
    }
    fn e214(&self) -> String {
        return format!("E[214]: Illegal operand format: format is [symbol] or [symbol, X]");
    }
    fn e301(&self) -> String {
        return format!("E[301]: code need to start with a legal START");
    }
    fn e302(&self) -> String {
        return format!("E[302]: BASE must be after LDB");
    }
    fn e303(&self) -> String {
        return format!("E[303]: The next instruction of LDB must be BASE");
    }
    fn e304(&self) -> String {
        return format!("E[304]: END must be the last instruction");
    }
    fn e305(&self, program_name: &str) -> String {
        return format!(
            "E[305]: {} is not a valid program name, maximum length of program name is 6",
            program_name
        );
    }
    fn e306(&self) -> String {
        return format!("E[306]: SIC/XE program is too large, maximum size is 1048576(2^20) bytes");
    }
    fn e307(&self) -> String {
        return format!("E[307]: Need to use BASE, but you haven't defined it");
    }
    fn e308(&self, base: &str) -> String {
        return format!("E[308]: Use BASE {} also out of range", base);
    }
    fn e309(&self) -> String {
        return format!("E[309]: Execute at unknown location");
    }
    fn e310(&self) -> String {
        return format!("E[310]: Invalid source code");
    }
    fn e311(&self, operand: &str) -> String {
        return format!("E[311]: Operand {} not found", operand);
    }
    fn e312(&self, operand: &str, base: &str) -> String {
        return format!("E[312]: Operand {} or base {} not found", operand, base);
    }
    fn e313(&self) -> String {
        return format!("E[313]: Operand and label can not be same");
    }
    fn e999(&self, msg: &str) -> String {
        return format!("E[999]: Assembler have bug: {}, please report it", msg);
    }
}

impl ErrMsg for ZH {
    fn e001(&self) -> String {
        return format!("E[001]: ????????????????????????");
    }
    fn e002(&self) -> String {
        return format!("E[002]: ??????????????????");
    }
    fn e003(&self, msg: &str) -> String {
        return format!("E[003]: ?????????????????????{}", msg);
    }
    fn e101(&self, symbol: &str) -> String {
        return format!("E[101]: ??????????????????: {}", symbol);
    }
    fn e102(&self, symbol: &str) -> String {
        return format!("E[102]: ?????? {} ????????????", symbol);
    }
    fn e103(&self, symbol: &str) -> String {
        return format!("E[103]: ?????? {} ??????????????????", symbol);
    }
    fn e104(&self, symbol: &str) -> String {
        return format!("E[104]: ?????? {} ?????????????????????", symbol);
    }
    fn e105(&self, symbol: &str) -> String {
        return format!("E[105]: ?????? {} ??????????????????", symbol);
    }
    fn e201(&self, mnemonic: &str) -> String {
        return format!("E[201]: ??????????????????: {}", mnemonic);
    }
    fn e202(&self) -> String {
        return format!("E[202]: ????????????????????????");
    }
    fn e203(&self, location: &str) -> String {
        return format!(
            "E[203]: ???????????????????????????{} ???????????? 16 ????????????",
            location
        );
    }
    fn e204(&self) -> String {
        return format!("E[204]: ?????????????????????????????????????????????????????? 1048576 (word ??? 3 ?????????)");
    }
    fn e205(&self) -> String {
        return format!("E[205]: ?????????????????????????????????????????????????????? -2048 ??? 2047");
    }
    fn e206(&self) -> String {
        // cspell:disable
        return format!("E[206]: ?????????????????????BYTE ????????? C'xxxx' ??? X'xxxx'");
        // cspell:enable
    }
    fn e207(&self) -> String {
        return format!("E[207]: BYTE ????????? ASCII ??????");
    }
    fn e208(&self) -> String {
        return format!("E[208]: BYTE ????????? 8 ?????????");
    }
    fn e209(&self) -> String {
        return format!("E[209]: BYTE X ?????????????????????????????????EX: F => 0F");
    }
    fn e210(&self, mnemonic: &str) -> String {
        return format!("E[210]: ????????? {} ??????????????????", mnemonic);
    }
    fn e211(&self, mnemonic: &str) -> String {
        return format!("E[211]: ????????? {} ???????????????", mnemonic);
    }
    fn e212(&self, register: &str) -> String {
        return format!("E[212]: ????????? {} ?????????", register);
    }
    fn e213(&self) -> String {
        return format!("E[213]: ??????????????????????????????????????? [?????????] ??? [?????????,?????????]");
    }
    fn e214(&self) -> String {
        return format!("E[214]: ??????????????????????????????????????? [??????] ??? [??????, X]");
    }
    fn e301(&self) -> String {
        return format!("E[301]: ???????????????????????? START ???????????????");
    }
    fn e302(&self) -> String {
        return format!("E[302]: BASE ????????? LDB ??????");
    }
    fn e303(&self) -> String {
        return format!("E[303]: LDB ????????????????????????????????? BASE");
    }
    fn e304(&self) -> String {
        return format!("E[304]: END ???????????????????????????");
    }
    fn e305(&self, program_name: &str) -> String {
        return format!(
            "E[305]: {} ???????????????????????????????????????????????????????????? 6",
            program_name
        );
    }
    fn e306(&self) -> String {
        return format!("E[306]: SIC/XE ?????????????????????????????? 1048576 (2^20) bytes");
    }
    fn e307(&self) -> String {
        return format!("E[307]: ???????????? BASE?????????????????????");
    }
    fn e308(&self, base: &str) -> String {
        return format!("E[308]: ?????? BASE {} ??????????????????", base);
    }
    fn e309(&self) -> String {
        return format!("E[309]: ????????????????????????");
    }
    fn e310(&self) -> String {
        return format!("E[310]: ?????????????????????");
    }
    fn e311(&self, operand: &str) -> String {
        return format!("E[311]: ????????? {} ?????????", operand);
    }
    fn e312(&self, operand: &str, base: &str) -> String {
        return format!("E[312]: ????????? {} ??? BASE {} ?????????", operand, base);
    }
    fn e313(&self) -> String {
        return format!("E[313]: ??????????????????????????????");
    }
    fn e999(&self, msg: &str) -> String {
        return format!("E[999]: {}, ???????????????", msg);
    }
}

impl EN {
    pub fn new() -> EN {
        EN
    }
}

impl ZH {
    pub fn new() -> ZH {
        ZH
    }
}

fn lang() -> &'static str {
    let lang: &str;

    match env::var("LANG") {
        Ok(val) => {
            lang = {
                if val.contains("zh") {
                    "zh"
                } else {
                    "en"
                }
            }
        }
        Err(_) => lang = "en",
    }

    return lang;
}

pub fn handler() -> Box<dyn ErrMsg> {
    let handler: Box<dyn ErrMsg>;

    match lang() {
        "zh" => {
            handler = Box::new(ZH::new());
        }
        "en" => {
            handler = Box::new(EN::new());
        }
        _ => {
            handler = Box::new(EN::new());
        }
    }

    return handler;
}
