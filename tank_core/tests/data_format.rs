use std::{fs::{self, File}, io::Read};
use tank_core::error::Result;


#[test]
fn filebytes_same()->Result<()>{
    let string=fs::read_to_string("../test_dep/no_new_line.in")?;

    let mut file=File::open("../test_dep/no_new_line.in")?;
    let mut vec=Vec::new();
    file.read_to_end(&mut vec)?;

    assert!(string.into_bytes()==vec);

    Ok(())
}