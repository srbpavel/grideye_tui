#[allow(unused)]
use crate::info;
#[allow(unused)]
use crate::warn;
#[allow(unused)]
use crate::error;

use crate::config::Config;

use crate::app::App;
use crate::app::Tab;
use crate::app::TabVariant;

use crate::mqtt;
//use crate::mqtt::Mqtt;
use crate::mqtt::CommonMsg;

use crate::ui;
use crate::ui::Device;
use crate::ui::UI_REFRESH_DELAY;

//use std::thread;
//use std::io;
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
use crossterm::event::Event;
use crossterm::event::KeyCode;

use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyEventState;
use crossterm::event::KeyModifiers;

/* // KEY Pause
use crossterm::event::KeyboardEnhancementFlags;
use crossterm::event::PushKeyboardEnhancementFlags;
use crossterm::event::PopKeyboardEnhancementFlags;
*/

//#[derive(PartialEq)]
//#[derive(Debug, PartialEq)]
#[derive(Debug, PartialEq, Clone)]
pub enum Device2Tab {
    Dynamic(String),
    Fixed(String),
}

impl Device2Tab {
    //
    pub fn get_tab(self) -> Tab {
        match self {
            Self::Dynamic(name) => Tab {
                name,
                variant: TabVariant::Dynamic,
            },
            Self::Fixed(name) => Tab {
                name,
                variant: TabVariant::Fixed,
            },
        }
    }
}

type DevicesKey = String;
pub type Devices = std::collections::HashMap <DevicesKey, Device>;
type DevicesToRemove = Vec<DevicesKey>;

//
pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
/*
pub fn run<W>(config: Config,
              write_logger: &simplelog::WriteLogger<W>,
) -> Result<(), Box<dyn std::error::Error>>
where
        W: std::io::Write + core::marker::Send + 'static,
{
*/

    // SETUP TERMINAL
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout,
             EnterAlternateScreen,
             EnableMouseCapture,
             /*
             // try harder! -> not working for Pause yet
             //
             // https://docs.rs/crossterm/latest/crossterm/event/enum.KeyCode.html#variant.Pause
             PushKeyboardEnhancementFlags(
                 KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
             ),
             */
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(config);

    /* // Record
    let record = log::Record::builder()
        .args(format_args!("Info -> in RUN"))
        .level(log::Level::Info)
        .build();

    write_logger.log(&record);
    */
    
    // LOOP
    let res = run_app(&mut terminal,
                      app,
    );
    
    // RESTORE TERMINAL
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        //PopKeyboardEnhancementFlags,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }
    
    Ok(())
}

//
fn run_app<B: Backend>(terminal: &mut Terminal<B>,
                       app: App,
) -> std::io::Result<()> {
    // LAUNCH MEASUREMENT THREAD
    let (data_sender, data_receiver) = mpsc::channel::<mqtt::Payload>();
    // COMMON LOG
    let (common_sender, common_receiver) = mpsc::channel::<CommonMsg>();
    
    // INCOMING DATA
    let mqtt = mqtt::Mqtt::new(app.config.clone()).connect();
    mqtt
        .subscribe()
        // THREAD SPAWN
        // -> incomming packet
        // --> listen for: payload + error_log
        .parse(data_sender,
               common_sender.clone(),
        );

    // SCREEN
    let mut render = ui::Render::new(app);
    
    // App.tabs.titles
    //
    // first tab - static -> playground
    render.app.push_title(
        Tab::new(String::from("color"),
                 TabVariant::Static,
        )
    );
    // second tab - static -> all heat_maps
    render.app.push_title(
        Tab::new(String::from("heatmap"),
                  TabVariant::Static,
        )
    );
    // third tab -> fixed in dynamic -> commomn_log 
    let mqtt_topic_error = mqtt::create_topic(render.app.config.mqtt_topic_base,
                                              &[render.app.config.mqtt_topic_error_suffix],
    );

    //
    let fixed_tab = Device2Tab::Fixed(mqtt_topic_error.clone());
    
    render.insert_device(fixed_tab);
    /*
    // 
    render.dynamic_tabs.push(fixed_tab);

    //
    render.app.push_title(Tab {
        name: mqtt_topic_error,
        variant: TabVariant::Fixed,
    });
    */
    
    loop {
        // on PAUSE we stop receive mqtt
        // verify retention ???
        if render.app.should_pause.eq(&false) {
            // Payload via channel from incomming mqtt
            for channel_data in data_receiver.try_iter() {
                channel_data.parse(&render.app.config,
                                   &mut render.devices,
                );
            }
            
            let devices_to_remove = devices_task(&mut render,
                                                 common_sender.clone(),
            );
            
            if let Some(devices_to_remove) = devices_to_remove {
                // REMOVE INACTIVE DEVICES        
                remove_inactive_devices(devices_to_remove,
                                        &mut render,
                                        common_sender.clone(),
                );
            };
        } else {
            render.devices
                .iter_mut()
                .for_each(|(_key, device)| {
                    device.status = ui::Status::OnPause;
                });
        }

        // COMMON_LOG
        for common_msg in common_receiver.try_iter() {
            // add msg to vec_deque
            render.common_log.add(common_msg);
            // shrink to limit size
            render.common_log.truncate();
        }

        // RENDER
        terminal.draw(|frame| {
            render.draw(frame);
        })?;
        
        // SLEEP
        // todo! test via timer
        std::thread::sleep(UI_REFRESH_DELAY);

        // KEY
        if crossterm::event::poll(UI_REFRESH_DELAY)? {
            if let Event::Key(key) = event::read()? {
                // just to match field "code"
                match key.code {
                    KeyCode::Char(c) => render.app.on_key(c),
                    KeyCode::Left | KeyCode::Backspace => render.app.on_left(),
                    KeyCode::Right => render.app.on_right(),
                    KeyCode::Esc => {
                        render.app.should_quit = true;
                    },
                    /* // try harder -> not working yet
                    KeyCode::Pause => {
                        app.on_pause();
                    },
                    */
                    _ => {}
                }

                // with modifiers as CONTROL/ALT/...
                /*
                match key {
                    KeyEvent {
                        code: KeyCode::Char(any_char),
                        modifiers: KeyModifiers::CONTROL,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    } => render.app.on_ctrl_key(any_char),
                    _ => {}

                }
                */

                if let KeyEvent {
                    code: KeyCode::Char(any_char),
                    modifiers: KeyModifiers::CONTROL,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
                } = key { render.app.on_ctrl_key(any_char) }
            }
        }

        if render.app.should_quit {
            return Ok(());
        }
    }
}

//
fn remove_inactive_devices(items: DevicesToRemove,
                           render: &mut ui::Render,
                           common_sender: mpsc::Sender<CommonMsg>,
) {
    //info!("remove_inactive_items(): {:?}", items);
    common_sender
        .send(
            CommonMsg::record(
                format!("remove_inactive_items(): {:?}",
                        items,
                )
            )
        )
        .unwrap();
    
    items
        .iter()
        .for_each(|device| {
            /*
            if let Some(_) = render.devices.remove(device) {
                render.remove_device(device);

                //display msg in log
                common_sender
                    .send(
                        CommonMsg::record(format!("device was deleted: {}", device))
                    )
                    .unwrap();
            };
            */
            if render.devices.remove(device).is_some() {
                render.remove_device(device);

                //display msg in log
                common_sender
                    .send(
                        CommonMsg::record(format!("device was deleted: {}", device))
                    )
                    .unwrap();
            };
        });
}

//
fn devices_task(render: &mut ui::Render,
                common_sender: mpsc::Sender<CommonMsg>,
) -> Option<DevicesToRemove> {
    let mut devices_to_remove = vec!();

    /* // DEBUG too many
    error!("render.dynamic_tabs: {:?}",
           render.dynamic_tabs,
    );
    */

    if !render.devices.is_empty() {
        render
            .devices
            .iter_mut()
            .for_each(|(key, single_device)| {
                // topic name which is used to get from hash_map
                //let device_tab_name = format!("{key}");
                let device_tab_name = String::from(key);
                
                /* // DEBUG too many
                error!("device: {:?} / {:?}",
                       device_tab_name,
                       single_device.status,
                );
                */
                
                // insert new active topic
                //
                // this verify device_name/mqtt topic
                if !render.dynamic_tabs.contains(&Device2Tab::Dynamic(device_tab_name.clone())) {

                    common_sender
                        .send(
                            CommonMsg::record(
                                format!("device_task(): --> new item -> {:?}",
                                        device_tab_name,
                                )
                            )
                        )
                        .unwrap();

                    /* // todo(!) try harder
                    &render.insert_device(
                        Device2Tab::Dynamic(device_tab_name)
                    );
                    */
                    
                    // /*
                    // mqtt topic name as key in hashmap
                    render
                        .dynamic_tabs
                        .push(Device2Tab::Dynamic(device_tab_name.clone())
                        );

                    // tab name
                    render
                        .app
                        .push_title(
                            Tab::new(device_tab_name,
                                     TabVariant::Dynamic,
                            )
                        );
                    // */
                    
                    // some work on existing device
                    //
                    // prepare devices to be deleted
                    } else {
                    /* // DEBUG too many
                    info!("devices_task() !contains: {:?}",
                          single_device.topic,
                    );
                    */
                    
                    match single_device.status {
                        ui::Status::OnPause => {},
                        _ => {
                            single_device.verify_status(&render.app.config,
                                                        &mut devices_to_remove,
                                                        //common_sender.clone(),
                            );
                        },
                    }
                    
                };
            });
    };

    if devices_to_remove.is_empty() {
        None
    } else {
        Some(devices_to_remove)
    }
}
