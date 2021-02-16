use std::str::FromStr;

use rhai::{Engine, RegisterFn};

use crate::error::{Error, Result};

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

impl ByteScannerScriptBinder {
    pub fn new(scanner: ByteScanner) -> Self {
        Self {
            scanner,
            err: Vec::new(),
        }
    }

    pub fn bind_to(engine: &mut Engine) -> &mut Engine {
        engine
            .register_type::<Self>()
            .register_fn("peek", Self::peek)
            .register_fn("rbyte", Self::rbyte)
            .register_fn("rblock", Self::rblock)
            .register_fn("rstr", Self::rstr)
            .register_fn("rint", Self::rint)
            .register_fn("ebyte", Self::ebyte)
            .register_fn("espace", Self::espace)
            .register_fn("eeoln", Self::eeoln)
            .register_fn("eeof", Self::eeof)
            .register_fn("estr", Self::estr)
            .register_fn("eint", Self::eint);

        engine
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
    fn rint(&mut self) -> i64 {
        match self.scanner.read::<i64>() {
            Ok(x) => x,
            Err(err) => {
                self.err.push(format!("{:?}", err));
                0
            }
        }
    }
    fn ebyte(&mut self, test: u8) -> bool {
        self.scanner.expect_byte(test)
    }
    fn espace(&mut self) -> bool {
        self.scanner.expect_space()
    }
    fn eeoln(&mut self) -> bool {
        self.scanner.expect_eoln()
    }
    fn eeof(&mut self) -> bool {
        self.scanner.expect_eof()
    }
    fn estr(&mut self, test: &str) -> bool {
        self.scanner.expect_str(test)
    }
    fn eint(&mut self, test: i64) -> bool {
        self.rint() == test
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
        ByteScannerScriptBinder::bind_to(&mut engine);

        let mut scope = Scope::new();
        scope.push(
            "input",
            ByteScannerScriptBinder::new(ByteScanner::from_bytes("1 2".to_string().into_bytes())),
        );

        engine
            .eval_with_scope::<bool>(
                &mut scope,
                r#"
                input.eint(1);
                input.espace();
                input.eint(2);
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
