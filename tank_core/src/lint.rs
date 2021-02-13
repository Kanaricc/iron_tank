use std::collections::HashMap;

pub struct DataLinter<'a> {
    linters: HashMap<&'a str, Box<dyn Fn(&Vec<u8>) -> Vec<String>>>,
}

impl<'a> DataLinter<'a> {
    pub fn new() -> Self {
        Self {
            linters: HashMap::new(),
        }
    }

    pub fn load_default_linter(&mut self) {
        // unexpected bytes.
        // bytes below 32 are all special chars defined in ASCII. Normally, data will not need them, except 10(NL) and 14(CR).
        // TODO: support other encode.
        self.linters.insert(
            "unexpected-bytes",
            Box::new(|bytes| {
                let warns:Vec<String> = bytes.iter().filter(|i| i < &&32u8)
                .filter(|i| i != &&10u8)
                .map(|i| format!("unexpected byte: {}", i))
                .collect();

                warns
            }),
        );

        // TODO: consecutive empty lines
        // TODO: start with empty line
        // TODO: extra spaces after lines
        // TODO: consecutive spaces
    }
}
