use std::{error, io};

pub fn disconnect(
    w: &mut impl io::Write,
    r: &mut impl io::Read,
    force: &bool,
    aliases: &Option<Vec<String>>,
) -> Result<(), Box<dyn error::Error>> {
    todo!()
}
