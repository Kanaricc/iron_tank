use std::str::FromStr;

use crate::error::{Error, Result};

pub struct ByteScanner<'a> {
    pointer: usize,
    data: &'a Vec<u8>,
}

impl<'a> ByteScanner<'a> {
    pub fn from_bytes(data: &'a Vec<u8>) -> Self {
        Self { pointer: 0, data }
    }

    pub fn peek(&self) -> Option<u8> {
        self.data.get(self.pointer).map(|f| f.clone())
    }

    pub fn expect_byte(&mut self, test: u8) -> bool {
        let res = match self.data.get(self.pointer) {
            Some(x) => x == &test,
            None => false,
        };

        if res {
            self.pointer += 1;
        }
        res
    }

    pub fn expect_string(&mut self, test: &str) -> bool {
        for chr in test.bytes() {
            if !self.expect_byte(chr) {
                return false;
            }
        }
        return true;
    }

    pub fn expect_integer(&mut self, test: i64) -> bool {
        self.expect_string(test.to_string().as_str())
    }

    pub fn expect_space(&mut self) -> bool {
        self.expect_byte(32u8)
    }

    pub fn expect_eoln(&mut self) -> bool {
        self.expect_byte(10u8)
    }

    pub fn expect_eof(&mut self)->bool{
        matches!(self.read_byte().unwrap(),None)
    }

    pub fn read_byte(&mut self) -> Result<Option<u8>> {
        let res = self.data.get(self.pointer).map(|f| f.clone());
        if let Some(_) = res {
            self.pointer += 1;
        }
        Ok(res)
    }

    /// Read a *block* from bytes, stopping when it faces a blank, even at the beginning.
    pub fn read_block(&mut self) -> Result<Vec<u8>> {
        let mut res = Vec::new();
        while let Some(chr) = self.peek() {
            if is_blank(chr) {
                break;
            }

            self.read_byte()?;
            res.push(chr);
        }

        Ok(res)
    }

    pub fn read_str(&mut self)->Result<String>{
        let res=String::from_utf8(self.read_block()?)?;
        Ok(res)
    }

    pub fn read<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        let block = self.read_block()?;
        let block = String::from_utf8(block)?;
        match block.parse::<T>() {
            Ok(x) => Ok(x),
            Err(_) => Err(Error::UnexpectedBlockType),
        }
    }
}

fn is_blank(chr: u8) -> bool {
    chr == 10u8 || chr == 32u8
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn normal()->Result<()>{
        let data="1 2\n3 str".to_string().into_bytes();
        let mut scanner=ByteScanner::from_bytes(&data);
        assert_eq!(scanner.read::<i32>()?,1);
        assert!(scanner.expect_space());
        assert_eq!(scanner.read::<i32>()?,2);
        assert!(scanner.expect_eoln());
        assert_eq!(scanner.read::<i32>()?,3);
        assert!(scanner.expect_space());
        assert_eq!(scanner.read_str()?,"str");
        assert!(scanner.expect_eof());

        Ok(())
    }

    #[test]
    #[should_panic]
    fn too_many_spaces(){
        let data="1 2\n 3 str".to_string().into_bytes();
        let mut scanner=ByteScanner::from_bytes(&data);
        assert_eq!(scanner.read::<i32>().unwrap(),1);
        assert!(scanner.expect_space());
        assert_eq!(scanner.read::<i32>().unwrap(),2);
        assert!(scanner.expect_eoln());
        assert_eq!(scanner.read::<i32>().unwrap(),3);
        assert!(scanner.expect_space());
        assert_eq!(scanner.read_str().unwrap(),"str");
        assert!(scanner.expect_eof());
    }

    #[test]
    fn bytes(){
        let data=vec![0u8,6u8,32u8];
        let mut scanner=ByteScanner::from_bytes(&data);

        assert!(scanner.expect_byte(0u8));
        assert!(scanner.expect_byte(6u8));
        assert!(scanner.expect_space());
    }

    #[test]
    #[should_panic]
    fn wrong_bytes(){
        let data=vec![0u8,6u8,10u8];
        let mut scanner=ByteScanner::from_bytes(&data);
        assert_eq!(scanner.read::<i32>().unwrap(),1);
    }


}