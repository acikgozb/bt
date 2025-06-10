use std::{error, io};

use crate::api::ScanArgs;

pub fn scan(f: &mut impl io::Write, args: &ScanArgs) -> Result<(), Box<dyn error::Error>> {
    todo!()
}
