use std::collections::HashMap;
use std::error::Error;
use std::cell::Cell;
use std::cell::RefCell;


pub struct SymbolTable {
    table: HashMap<String, SymbolData>,
}

pub struct SymbolData {
    address: Cell<u16>,
    need_alloc: Cell<bool>,
    wait_byte_code: RefCell<Vec<u16>>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
       return SymbolTable { table: HashMap::new() }
    }

    pub fn is_legal(&self, _symbol: &str) -> bool {
        return true;
    }

    pub fn get(&mut self, symbol: &str, address: u16) -> (u16, bool) {
       match self.table.get(symbol) {
            Some(data) => {
                if data.need_alloc.get() {
                    data.wait_byte_code.borrow_mut().push(address);
                }
                return (data.address.get(), data.need_alloc.get());
            }
            None => {
                self.insert_without_address(symbol, address);
                return (0x0, true);
            }
       }
    }

    fn insert_without_address(&mut self, symbol: &str, address: u16) {
        let mut wait_byte_code = Vec::new();
        wait_byte_code.push(address);
        self.table.insert(symbol.to_string(), SymbolData { 
            address: Cell::new(0x0),
            need_alloc: Cell::new(true),
            wait_byte_code: RefCell::new(wait_byte_code),
        });
    }

    pub fn insert(&mut self, symbol: &str, address: u16) -> Result<Vec<u16>, Box<dyn Error>> {

        match self.table.get(symbol) {
            Some(symbol_data) => {
                if symbol_data.need_alloc.get() {
                    symbol_data.need_alloc.set(false);
                    symbol_data.address.set(address);
                    return Ok(symbol_data.wait_byte_code.borrow().clone());
                } else {
                    return Err(format!("duplicate symbol：{}", symbol).into());
                }
            },
            None => {
                self.table.insert(symbol.to_string(), SymbolData { 
                    address: Cell::new(address),
                    need_alloc: Cell::new(false),
                    wait_byte_code: RefCell::new(Vec::new()),
                });
                return Ok(Vec::new());
            }
        }
    }

    pub fn inter(&self) {
        self.table.iter().for_each(|(k, v)| {
            println!("{} -> {:04X}", k, v.address.get());
        });
    }
}