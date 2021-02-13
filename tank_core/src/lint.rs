use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

#[derive(Serialize, Deserialize)]
pub struct DataLinter<'a> {
    #[serde(skip_serializing, skip_deserializing)]
    _linters: HashMap<&'a str, Box<dyn Fn(&Vec<u8>) -> Vec<String>>>,
    linters: Vec<String>,
}

impl<'a> DataLinter<'a> {
    pub fn new(linters: Vec<&str>) -> Result<Self> {
        let mut v = Self {
            _linters: HashMap::new(),
            linters: linters.into_iter().map(|f| f.to_string()).collect(),
        };

        v.load_default_linter();
        v.check_config()?;
        Ok(v)
    }

    fn from_string(content: &str) -> Result<Self> {
        let v: Self = serde_yaml::from_str(&content).unwrap();
        Ok(v)
    }
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).unwrap();
        let mut v = Self::from_string(&content)?;

        v.load_default_linter();
        v.check_config()?;
        Ok(v)
    }

    pub fn check_config(&self) -> Result<()> {
        assert!(self._linters.len() > 0);
        for key in self.linters.iter() {
            if !self._linters.contains_key(key.as_str()) {
                return Err(Error::Argument(format!("unknown linter: {}", key)));
            }
        }
        Ok(())
    }

    pub fn load_default_linter(&mut self) {
        // unexpected bytes.
        // bytes below 32 are all special chars defined in ASCII. Normally, data will not need them, except 10(NL) and 14(CR).
        // TODO: support other encode.
        self._linters.insert(
            "unexpected-bytes",
            Box::new(|bytes: &Vec<u8>| {
                let warns: Vec<String> = bytes
                    .iter()
                    .filter(|i| i < &&32u8)
                    .filter(|i| i != &&10u8)
                    .map(|i| format!("unexpected byte: {}.", i))
                    .collect();

                warns
            }),
        );

        // consecutive empty lines
        self._linters.insert(
            "consecutive-empty-lines",
            Box::new(|bytes: &Vec<u8>| {
                for i in 1..bytes.len() {
                    if bytes[i] == bytes[i - 1] && bytes[i] == 10u8 {
                        return vec![format!("consecutive empty lines.")];
                    }
                }

                return vec![];
            }),
        );

        // start with empty line
        self._linters.insert(
            "start-with-empty-line",
            Box::new(|bytes: &Vec<u8>| {
                if let Some(chr) = bytes.first() {
                    if chr == &10u8 {
                        return vec![format!("start with empty line.")];
                    }
                }

                return vec![];
            }),
        );

        // extra spaces after lines
        self._linters.insert(
            "extra-spaces-after-lines",
            Box::new(|bytes: &Vec<u8>| {
                for i in 1..bytes.len() {
                    if bytes[i - 1] == 10u8 && bytes[i] == 32u8 {
                        return vec![format!("extra spaces after lines")];
                    }
                }

                return vec![];
            }),
        );

        // consecutive spaces
        self._linters.insert(
            "consecutive-spaces",
            Box::new(|bytes: &Vec<u8>| {
                for i in 1..bytes.len() {
                    if bytes[i - 1] == 32u8 && bytes[i] == 32u8 {
                        return vec![format!("consecutive spaces")];
                    }
                }

                return vec![];
            }),
        );
    }

    pub fn check(&self, bytes: &Vec<u8>) -> Vec<String> {
        let mut res = vec![];
        for key in self.linters.iter() {
            res.extend(self._linters.get(key.as_str()).unwrap()(bytes));
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_KEYS: [&str; 5] = [
        "unexpected-bytes",
        "consecutive-empty-lines",
        "start-with-empty-line",
        "extra-spaces-after-lines",
        "consecutive-spaces",
    ];
    #[test]
    fn safe_data() -> Result<()> {
        let sample = "1 3\n2\n".to_string().into_bytes();

        let mut linter = DataLinter::new(ALL_KEYS.to_vec())?;
        linter.load_default_linter();
        assert_eq!(linter.check(&sample).len(), 0);

        Ok(())
    }

    #[test]
    fn too_many_spaces() -> Result<()> {
        let sample = "1  3\n2\n".to_string().into_bytes();

        let mut linter = DataLinter::new(ALL_KEYS.to_vec())?;
        linter.load_default_linter();

        assert_eq!(vec!["consecutive spaces"], linter.check(&sample));

        Ok(())
    }

    #[test]
    fn empty_line_at_beginning() -> Result<()> {
        let sample = "\n1 3\n2\n".to_string().into_bytes();

        let mut linter = DataLinter::new(ALL_KEYS.to_vec())?;
        linter.load_default_linter();

        assert_eq!(vec!["start with empty line."], linter.check(&sample));
        Ok(())
    }

    #[test]
    fn too_many_empty_line() -> Result<()> {
        let sample = "1 3\n\n2\n".to_string().into_bytes();

        let mut linter = DataLinter::new(ALL_KEYS.to_vec())?;
        linter.load_default_linter();

        assert_eq!(vec!["consecutive empty lines."], linter.check(&sample));
        Ok(())
    }

    #[test]
    fn unexpected_chars() -> Result<()> {
        let sample = "1 3\r\n2\n".to_string().into_bytes();

        let mut linter = DataLinter::new(ALL_KEYS.to_vec())?;
        linter.load_default_linter();

        assert_eq!(vec!["unexpected byte: 13."], linter.check(&sample));
        Ok(())
    }
}
