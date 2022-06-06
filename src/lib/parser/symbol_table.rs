use regex::Regex;
use std::collections::HashMap;

#[path = "../err.rs"]
mod err;

pub struct SymbolData {
    location: u32,
    need_alloc: bool,
    waiting_list: Vec<u32>,
}

pub struct SymbolTable {
    table: HashMap<String, SymbolData>,
    legal_symbol_regex: Regex,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            table: HashMap::new(),
            legal_symbol_regex: Regex::new(r"^[A-Z][A-Z0-9]*$").unwrap(),
        }
    }

    pub fn remove_waiting(&mut self, symbol: &str, location: u32) {
        let data = self.table.get_mut(symbol).unwrap();
        data.waiting_list.retain(|&x| x != location);
    }

    pub fn is_legal(&self, symbol: &str) -> bool {
        return self.legal_symbol_regex.is_match(symbol);
    }

    pub fn get_location(&self, symbol: &str) -> Option<(u32, bool)> {
        if self.table.contains_key(symbol) {
            let data = self.table.get(symbol).unwrap();
            return Some((data.location, data.need_alloc));
        } else {
            return None;
        }
    }

    pub fn get_location_or_create(
        &mut self,
        symbol: &str,
        obj_code_location: u32,
    ) -> Result<(u32, bool), String> {
        if !self.is_legal(symbol) {
            return Err(err::handler().e101(symbol));
        }
        match self.table.get_mut(symbol) {
            Some(data) => {
                if data.need_alloc {
                    data.waiting_list.push(obj_code_location);
                }
                return Ok((data.location, data.need_alloc));
            }
            None => {
                self.no_location_insert(symbol, obj_code_location);
                return Ok((0x0, true));
            }
        }
    }

    pub fn insert(&mut self, symbol: &str, obj_code_location: u32) -> Result<Vec<u32>, String> {
        if !self.is_legal(symbol) {
            return Err(err::handler().e101(symbol));
        }

        match self.table.get_mut(symbol) {
            Some(symbol_data) => {
                if symbol_data.need_alloc {
                    symbol_data.need_alloc = false;
                    symbol_data.location = obj_code_location;
                    return Ok(symbol_data.waiting_list.clone());
                } else {
                    return Err(err::handler().e102(symbol));
                }
            }
            None => {
                self.have_location_insert(symbol, obj_code_location);
                return Ok(vec![]);
            }
        }
    }

    pub fn inter(&self) {
        self.table.iter().for_each(|(k, v)| {
            println!("{} -> {:04X} : {:?}", k, v.location, v.waiting_list);
        });
    }

    fn no_location_insert(&mut self, symbol: &str, obj_code_location: u32) {
        self.table.insert(
            symbol.to_string(),
            SymbolData {
                location: 0,
                need_alloc: true,
                waiting_list: vec![obj_code_location],
            },
        );
    }

    fn have_location_insert(&mut self, symbol: &str, obj_code_location: u32) {
        self.table.insert(
            symbol.to_string(),
            SymbolData {
                location: obj_code_location,
                need_alloc: false,
                waiting_list: vec![],
            },
        );
    }
}
