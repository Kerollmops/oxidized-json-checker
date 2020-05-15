use std::io;
use oxidized_json_checker::JsonChecker;

fn main() -> io::Result<()> {
    let mut checker = JsonChecker::new(io::stdin());
    io::copy(&mut checker, &mut io::sink())?;
    checker.finish()?;
    eprintln!("Seems good.");
    Ok(())
}
