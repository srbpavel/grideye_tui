#[allow(unused)]
use log::info;
#[allow(unused)]
use log::warn;
#[allow(unused)]
use log::error;

//use log::Log;

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

    /* // OLD
    // we cannot have it together !!!
    env_logger::init();
    info!("GRIDEYE_TERMINAL");
    */

    /* // Record
    let write_logger = //simplelog::SimpleLogger::new(
        simplelog::WriteLogger::new(
            simplelog::LevelFilter::Info,
            simplelog::Config::default(),
            std::fs::File::create(config.log_file).unwrap(),
        );

    let record = log::Record::builder()
        .args(format_args!("Info!"))
        .level(log::Level::Info)
        .build();
    
    write_logger.log(&record);
    */
    
    // /* // NEW
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
                //std::fs::File::create("my_rust_binary.log").unwrap(),
                // todo(!) as fs can be ready only/full/...
                std::fs::File::create(config.log_file).unwrap(), // niet goed 
            ),
        ]
    ).unwrap();
    info!("GRIDEYE_TERMINAL");
    // */
    
    //let config = config::CONFIG;

    /*
    info!("config.now() '{:?}'",
          config.kolik::<chrono::Local>(
              chrono::Local::now(),
          ),
    );
    
    info!("cfg_param: '{}' -> value: '{:?}'",
          config.datetime_timezone,
          config.ted_hned(),
    );
    */

    run::run(config)?;
    //run::run(config, &write_logger)?;

    Ok(())
}
