use crate::{
    byte_scanner::{ByteScanner, ByteScannerScriptBinder, ScriptInject},
    error::{Error, Result},
};
use rhai::{Engine, Scope,AST};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

pub const ALL_KEYS: [&str; 5] = [
    "unexpected-bytes",
    "consecutive-empty-lines",
    "start-with-empty-line",
    "extra-spaces-after-lines",
    "consecutive-spaces",
];
#[derive(Serialize, Deserialize)]
pub struct DataLinter<'a> {
    linters: Vec<String>,
    #[serde(skip_serializing, skip_deserializing)]
    _linters: HashMap<&'a str, Box<dyn Fn(&Vec<u8>) -> Vec<String>>>,

    #[serde(rename = "customLints")]
    custom_lints: Option<Vec<String>>,

    #[serde(skip_serializing, skip_deserializing)]
    _custom_lints:Vec<AST>,
}

impl<'a> DataLinter<'a> {
    pub fn new(linters: Vec<&str>) -> Result<Self> {
        let mut v = Self {
            _linters: HashMap::new(),
            linters: linters.into_iter().map(|f| f.to_string()).collect(),
            custom_lints: None,
            _custom_lints: Vec::new(),
        };

        v.init()?;
        Ok(v)
    }

    fn from_string(content: &str) -> Result<Self> {
        let mut v: Self = serde_yaml::from_str(&content).unwrap();
        v.init()?;
        Ok(v)
    }
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).unwrap();
        let v = Self::from_string(&content)?;
        
        Ok(v)
    }

    /// this function is used to initialize itself when loading from config
    /// TODO: this should be avoid.
    pub(crate) fn init(&mut self) -> Result<()> {
        self.load_default_linter();
        self.check_config()?;
        self.compile_scripts()?;

        Ok(())
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

    fn compile_scripts(&mut self) ->Result<()>{
        let engine=Engine::new();
        if let Some(sources)=&self.custom_lints{
            for source in sources.iter(){
                let t=engine.compile(source)?;
                self._custom_lints.push(t);
            }
        }

        Ok(())
    }

    pub fn push_custom_lint(&mut self, script: String)->Result<()> {
        self._custom_lints.push(Engine::new().compile(&script)?);

        Ok(())
    }

    pub fn load_default_linter(&mut self) {
        // unexpected bytes.
        // bytes below 32 are all special chars defined in ASCII. Normally, data does not need them, except 10(LF) and 14(CR).
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
                        return vec![format!("consecutive spaces.")];
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

            for script in &self._custom_lints {
                let scanner = ByteScanner::from_bytes(bytes.clone());
                let binder = ByteScannerScriptBinder::new(scanner);
                let mut engine = Engine::new();
                let mut scope = Scope::new();
                binder.inject_engine(&mut engine);
                binder.inject_scope(&mut scope, "data");

                engine.eval_ast_with_scope::<i64>(&mut scope, script).unwrap();

                let binder: ByteScannerScriptBinder = scope.get_value("data").unwrap();
                let script_res = binder.get_err();
                if !script_res.is_empty() {
                    res.extend(script_res.into_iter());
                }
            }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() -> Result<()> {
        let linter = DataLinter::new(ALL_KEYS.to_vec())?;
        let linter = serde_yaml::to_string(&linter).unwrap();
        println!("{}", linter);

        Ok(())
    }
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

        assert_eq!(vec!["consecutive spaces."], linter.check(&sample));

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

    #[test]
    fn custom_lint_ok() -> Result<()> {
        let sample = "1 3\n2".to_string().into_bytes();

        let mut linter = DataLinter::new(ALL_KEYS.to_vec())?;
        linter.load_default_linter();
        linter.push_custom_lint(
            r#"
            data.ri64();
            data.espace();
            data.ri64();
            data.eeoln();
            data.ri64();
            data.eeof();
            0
                    "#
            .to_string(),
        )?;
        assert_eq!(linter.check(&sample).len(), 0);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn custom_lint_err() {
        let sample = "1 3\n2\n".to_string().into_bytes();

        let mut linter = DataLinter::new(ALL_KEYS.to_vec()).unwrap();
        linter.load_default_linter();
        linter.push_custom_lint(
            r#"
            data.ri64();
            data.espace();
            data.ri64();
            data.eeoln();
            data.ri64();
            data.eeof();
            0
                    "#
            .to_string(),
        ).unwrap();
        assert_eq!(linter.check(&sample).len(), 0);
    }
}
