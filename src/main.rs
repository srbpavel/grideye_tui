mod app;
mod mqtt;
mod run;
mod ui;

mod pallete;
//use pallete::PALLETE_RYGB_SORTED as PALLETE;
//use pallete::IRON_BOW as PALLETE;
use pallete::IRON_BOW_LONG as PALLETE;

//
fn main() -> Result<(), Box<dyn std::error::Error>> {
    run::run()?;

    Ok(())
}
