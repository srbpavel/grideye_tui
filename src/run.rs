use crate::app::App;
use crate::mqtt;
use crate::ui;
use crate::ui::Data;

use std::thread;
use std::io;
use std::sync::mpsc;

use tui::Terminal;
use tui::backend::Backend;
use tui::backend::CrosstermBackend;

use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::event;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::event::KeyCode;
use crossterm::event::Event;

//
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP TERMINAL
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout,
             EnterAlternateScreen,
             EnableMouseCapture,
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // LOOP
    let app = App::new("rust EPS32 + GridEye 8x8 --> TUI");
    let res = run_app(&mut terminal, app);
    
    // RESTORE TERMINAL
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }
    
    Ok(())
}

//
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    // LAUNCH MEASUREMENT THREAD
    let mut data_to_render = Data::default();
    let (data_sender, data_receiver) = mpsc::channel();

    // INCOMING DATA
    let mqtt = mqtt::Mqtt::connect();
    mqtt
        .subscribe()
        .parse(data_sender);
    
    loop {
        // OUTPUT DATA via channel from incomming mqtt
        for channel_data in data_receiver.try_iter() {
            data_to_render.fill(channel_data)
        }

        data_to_render.truncate();

        terminal.draw(|frame| {
            ui::draw(frame,
                     &mut app,
                     &mut data_to_render,
            )
        })?;
        
        // todo! test via timer
        thread::sleep(ui::UI_REFRESH_DELAY);
        
        // KEY
        // -> OLD
        //if ui::key_interupt().is_some() { return Ok(()) };
        // -> NEW
        if crossterm::event::poll(ui::UI_REFRESH_DELAY)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => app.on_key(c),
                    KeyCode::Left => app.on_left(),
                    KeyCode::Right => app.on_right(),
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
