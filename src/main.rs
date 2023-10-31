#[allow(unused)]
use log::info;
#[allow(unused)]
use log::warn;
#[allow(unused)]
use log::error;

mod config;
mod alarma;
mod app;
mod mqtt;
mod pallete;
mod run;
mod ui;

//use pallete::PALLETE_RYGB_SORTED as PALLETE;
//use pallete::IRON_BOW as PALLETE;
use pallete::IRON_BOW_LONG as PALLETE;

//
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::CONFIG;
    
    // todo! why not append?
    let log_file = std::fs::File::create(config.log_file)?;
    
    simplelog::CombinedLogger::init(
        vec![
            simplelog::TermLogger::new(
                //simplelog::LevelFilter::Warn,
                //simplelog::LevelFilter::Debug,
                //simplelog::LevelFilter::Info,
                simplelog::LevelFilter::Error,
                simplelog::Config::default(),
                simplelog::TerminalMode::Mixed,
                simplelog::ColorChoice::Auto,
            ),

            simplelog::WriteLogger::new(
                simplelog::LevelFilter::Info,
                simplelog::Config::default(),
                log_file,
            ),
        ]
    ).unwrap();
    info!("GRIDEYE_TERMINAL RATATUI");
    
    run::run(config)?;

    Ok(())
}
