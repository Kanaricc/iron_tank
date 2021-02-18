use std::{fmt::Display, str::FromStr};

use rhai::{Engine, RegisterFn, Scope};

use crate::error::{Error, Result};

pub trait ScriptInject {
    fn inject_engine(&self, engine: &mut Engine);
    fn inject_scope(self, scope: &mut Scope, key: &'static str);
}

#[derive(Clone, Debug)]
pub struct ByteScanner {
    pointer: usize,
    data: Vec<u8>,
}

impl ByteScanner {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { pointer: 0, data }
    }

    pub fn peek(&mut self) -> Option<u8> {
        self.data.get(self.pointer).map(|f| f.clone())
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        let res = self.data.get(self.pointer).map(|f| f.clone());
        if let Some(_) = res {
            self.pointer += 1;
        }
        res
    }

    /// Read a *block* from bytes, stopping when it faces *any* whitespace or control, even at the beginning,
    /// which means it does not ignore any of them.
    pub fn read_block(&mut self) -> Vec<u8> {
        let mut res = Vec::new();
        while let Some(x) = self.peek() {
            let chr = char::from(x);
            // break when entering *any* whitespace or control.
            if chr.is_ascii_whitespace() || chr.is_ascii_control() {
                break;
            }

            self.read_byte();
            res.push(x);
        }

        res
    }

    pub fn read_str(&mut self) -> Result<String> {
        let res = String::from_utf8(self.read_block())?;
        Ok(res)
    }

    pub fn read<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        let block = self.read_block();
        let block = String::from_utf8(block)?;
        match block.parse::<T>() {
            Ok(x) => Ok(x),
            Err(_) => Err(Error::UnexpectedBlockType),
        }
    }

    pub fn expect_byte(&mut self, test: u8) -> bool {
        match self.read_byte() {
            Some(x) => x == test,
            None => false,
        }
    }

    pub fn expect_bytes(&mut self, test: &Vec<u8>) -> bool {
        for x in test {
            let t = match self.read_byte() {
                Some(y) => x == &y,
                None => false,
            };
            if !t {
                return false;
            }
        }
        return true;
    }

    pub fn expect_space(&mut self) -> bool {
        self.expect_byte(32u8)
    }

    pub fn expect_eoln(&mut self) -> bool {
        self.expect_byte(10u8)
    }

    pub fn expect_eof(&mut self) -> bool {
        matches!(self.read_byte(), None)
    }

    pub fn expect_str(&mut self, test: &str) -> bool {
        self.read_block() == test.as_bytes().to_vec()
    }
}

#[derive(Clone)]
pub struct ByteScannerScriptBinder {
    scanner: ByteScanner,
    err: Vec<String>,
}

impl ScriptInject for ByteScannerScriptBinder {
    fn inject_engine(&self, engine: &mut Engine) {
        engine
            .register_type::<Self>()
            .register_fn("peek", Self::peek)
            .register_fn("rbyte", Self::rbyte)
            .register_fn("rblock", Self::rblock)
            .register_fn("rstr", Self::rstr)
            .register_fn("ri64", Self::read::<i64>)
            .register_fn("rf64", Self::read::<f64>)
            .register_fn("ebyte", Self::ebyte)
            .register_fn("espace", Self::espace)
            .register_fn("eeoln", Self::eeoln)
            .register_fn("eeof", Self::eeof)
            .register_fn("estr", Self::estr)
            .register_fn("ei64", Self::expect::<i64>);
    }

    fn inject_scope(self, scope: &mut Scope, key: &'static str) {
        scope.push(key, self);
    }
}

impl ByteScannerScriptBinder {
    pub fn new(scanner: ByteScanner) -> Self {
        Self {
            scanner,
            err: Vec::new(),
        }
    }

    pub fn get_err(&self) -> Vec<String> {
        self.err.clone()
    }
    fn peek(&mut self) -> u8 {
        self.scanner.peek().unwrap_or_else(|| {
            self.err.push(format!("peek error"));
            0
        })
    }
    fn rbyte(&mut self) -> u8 {
        self.scanner.read_byte().unwrap_or_else(|| {
            self.err.push(format!("read byte error"));
            0
        })
    }
    fn rblock(&mut self) -> Vec<u8> {
        self.scanner.read_block()
    }
    fn rstr(&mut self) -> String {
        match self.scanner.read_str() {
            Ok(str) => str,
            Err(err) => {
                self.err.push(format!("{:?}", err));
                String::new()
            }
        }
    }
    fn read<T: FromStr + Default>(&mut self) -> T {
        match self.scanner.read::<T>() {
            Ok(x) => x,
            Err(err) => {
                self.err.push(format!("{:?}", err));
                T::default()
            }
        }
    }
    fn ebyte(&mut self, test: u8) -> bool {
        if !self.scanner.expect_byte(test) {
            self.err.push(format!("expect `{}`.", test));
            return false;
        }
        return true;
    }
    fn espace(&mut self) -> bool {
        if !self.scanner.expect_space() {
            self.err.push(format!("expect `whitespace`."));
            return false;
        }
        return true;
    }
    fn eeoln(&mut self) -> bool {
        if !self.scanner.expect_eoln() {
            self.err.push(format!("expect `End of Line`."));
            return false;
        }
        return true;
    }
    fn eeof(&mut self) -> bool {
        if !self.scanner.expect_eof() {
            self.err.push(format!("expect `End of File`."));
            return false;
        }
        return true;
    }

    fn estr(&mut self, test: &str) -> bool {
        if !self.scanner.expect_str(test) {
            self.err.push(format!("expect string `{}`.", test));
            return false;
        }
        return true;
    }
    fn expect<T: FromStr + Default + Eq + Display>(&mut self, test: T) -> bool {
        if self.read::<T>() != test {
            self.err.push(format!("expect integer `{}`.", test));
            return false;
        }
        return true;
    }
}

#[cfg(test)]
mod tests {
    use rhai::{Engine, Scope};

    use super::*;

    #[test]
    fn normal() -> Result<()> {
        let data = "1 2\n3 str".to_string().into_bytes();
        let mut scanner = ByteScanner::from_bytes(data);
        assert_eq!(scanner.read::<i32>()?, 1);
        assert!(scanner.expect_space());
        assert_eq!(scanner.read::<i32>()?, 2);
        assert!(scanner.expect_eoln());
        assert_eq!(scanner.read::<i32>()?, 3);
        assert!(scanner.expect_space());
        assert_eq!(scanner.read_str()?, "str");
        assert!(scanner.expect_eof());

        Ok(())
    }

    #[test]
    #[should_panic]
    fn too_many_spaces() {
        let data = "1 2\n 3 str".to_string().into_bytes();
        let mut scanner = ByteScanner::from_bytes(data);
        assert_eq!(scanner.read::<i32>().unwrap(), 1);
        assert!(scanner.expect_space());
        assert_eq!(scanner.read::<i32>().unwrap(), 2);
        assert!(scanner.expect_eoln());
        assert_eq!(scanner.read::<i32>().unwrap(), 3);
        assert!(scanner.expect_space());
        assert_eq!(scanner.read_str().unwrap(), "str");
        assert!(scanner.expect_eof());
    }

    #[test]
    fn bytes() {
        let data = vec![0u8, 6u8, 32u8];
        let mut scanner = ByteScanner::from_bytes(data);

        assert!(scanner.expect_byte(0u8));
        assert!(scanner.expect_byte(6u8));
        assert!(scanner.expect_space());
    }

    #[test]
    #[should_panic]
    fn wrong_bytes() {
        let data = vec![0u8, 6u8, 10u8];
        let mut scanner = ByteScanner::from_bytes(data);
        assert_eq!(scanner.read::<i32>().unwrap(), 1);
    }

    #[test]
    fn bind_with_script() {
        let mut engine = Engine::new();
        let binder =
            ByteScannerScriptBinder::new(ByteScanner::from_bytes("1 2".to_string().into_bytes()));
        binder.inject_engine(&mut engine);
        let mut scope = Scope::new();
        binder.inject_scope(&mut scope, "input");

        engine
            .eval_with_scope::<bool>(
                &mut scope,
                r#"
                input.ei64(1);
                input.espace();
                input.ei64(2);
                input.eeof();
                "#,
            )
            .unwrap();
        assert_eq!(
            scope
                .get_value::<ByteScannerScriptBinder>("input")
                .unwrap()
                .get_err(),
            Vec::<String>::new()
        );
    }
}
