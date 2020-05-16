use std::io;
use oxidized_json_checker::JsonChecker;

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut checker = JsonChecker::new(stdin.lock());
    io::copy(&mut checker, &mut io::sink())?;
    checker.finish()?;
    eprintln!("Seems good.");
    Ok(())
}
