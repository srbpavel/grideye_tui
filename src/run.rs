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
use crate::mqtt::CommonMsg;

use crate::ui;
use crate::ui::Render;
use crate::ui::Device;

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use std::time::Duration;
use std::time::Instant;

use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::backend::CrosstermBackend;

use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::ExecutableCommand;

use crossterm::event;
use crossterm::event::Event as CrosstermEvent;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;

/* // KEY Pause
use crossterm::event::KeyboardEnhancementFlags;
use crossterm::event::PushKeyboardEnhancementFlags;
use crossterm::event::PopKeyboardEnhancementFlags;
*/

type Err = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Err>;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Tick,
    OnKey(char),
    Modifier(char),
    OnLeft,
    OnRight,
    Pause,
    Quit,
}

#[derive(Debug)]
pub struct EventHandler {
    //_sender: Sender<Event>,
    receiver: Receiver<Event>,
    //_handler: std::thread::JoinHandle<()>,
}

impl EventHandler {
    //
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);

        let (sender, receiver) = channel();

        let _handler = {
            //let sender = sender.clone();

            std::thread::spawn(move || {
                // TICK init
                let mut last_tick = Instant::now();

                // LOOP
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    // POLL
                    if event::poll(timeout)
                        .expect("no events available") {
                            match event::read().expect("unable to read event") {
                                CrosstermEvent::Key(key) => {
                                    if key.kind == event::KeyEventKind::Press {
                                        match key.code {
                                            KeyCode::Char(c) => {
                                                match key.modifiers {
                                                    KeyModifiers::CONTROL => sender.send(Event::Modifier(c)),
                                                    _ => sender.send(Event::OnKey(c)),
                                                }
                                            },
                                            KeyCode::Left | KeyCode::Backspace => sender.send(Event::OnLeft),
                                            KeyCode::Right => sender.send(Event::OnRight),
                                            KeyCode::Esc => sender.send(Event::Quit),
                                            KeyCode::Pause => sender.send(Event::Pause),
                                            _ => Ok(()),
                                        }
                                    } else {
                                        Ok(())
                                    }
                                },
                                //_ => unimplemented!(),
                                _ => Ok(()),
                            }
                            .expect("failed to send terminal event")
                        }

                    // UDPATE LAST TICK since previous tick
                    if last_tick.elapsed() >= tick_rate {
                        sender.send(Event::Tick)
                            .expect("failed to send tick event");

                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self {
            //sender: sender,
            receiver,
            //_handler: handler,
        }
    }
    
    //
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}

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
pub fn run(config: Config) -> Result<()> {
    // SETUP TERMINAL
    startup()?;
    
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    
    let app = App::new(config);

    // LOOP
    let res = run_app(&mut terminal,
                      app,
    );
    
    // RESTORE TERMINAL
    shutdown()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }
    
    Ok(())
}

//
fn run_app<B: Backend>(terminal: &mut Terminal<B>,
                       app: App,
) -> Result<()>
where
    B: Backend
{
    // LAUNCH MEASUREMENT THREAD
    let (data_sender, data_receiver) = channel::<mqtt::Payload>();
    // COMMON LOG
    let (common_sender, common_receiver) = channel::<CommonMsg>();
    
    // INCOMING DATA
    mqtt::Mqtt::new(app.config.clone())
        .connect()
        .subscribe()
        // THREAD SPAWN
        // -> incomming packet
        // --> listen for: payload + error_log
        .parse(data_sender,
               common_sender.clone(),
        );
    
    // SCREEN
    let mut render = Render::new(app);
    
    // App.tabs.titles
    // + first tab - static -> playground
    render.app.push_title(
        Tab::new(String::from("color"),
                 TabVariant::Static,
        )
    );
    // + second tab - static -> all heat_maps
    render.app.push_title(
        Tab::new(String::from("heatmap"),
                 TabVariant::Static,
        )
    );
    // + third tab -> fixed in dynamic -> commomn_log 
    let mqtt_topic_error = mqtt::create_topic(render.app.config.mqtt_topic_base,
                                              &[render.app.config.mqtt_topic_error_suffix],
    );

    // + common_log
    let fixed_tab = Device2Tab::Fixed(mqtt_topic_error.clone());
    render.insert_device(fixed_tab);

    let events = EventHandler::new(render.app.config.delay_ui_refresh);

    loop {
        // ON_PAUSE we stop receive mqtt
        // verify retention ???
        if render.app.should_pause.eq(&false) {
            // Payload via channel from incomming mqtt
            for channel_data in data_receiver.try_iter() {
                // update or init device data
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

        /*
        // OBS
        //here we will pasuse if temperature negative
        if negative_pause.eq(&true) {
            render.app.on_pause();
        }
        //_
        */
        
        // KEY ACTION + TICK
        match events.next()? {
            Event::Tick => {},
            Event::OnKey(c) => render.app.on_key(c),
            Event::Modifier(c) => render.app.on_ctrl_key(c),
            Event::OnLeft => render.app.on_left(),
            Event::OnRight => render.app.on_right(),
            Event::Pause => render.app.on_pause(),
            Event::Quit => render.app.should_quit = true,
        }
        
        // EXIT
        if render.app.should_quit {
            return Ok(());
        }
    }
}

//
fn remove_inactive_devices(items: DevicesToRemove,
                           render: &mut Render,
                           common_sender: Sender<CommonMsg>,
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
fn devices_task(render: &mut Render,
                common_sender: Sender<CommonMsg>,
) -> Option<DevicesToRemove> {
    let mut devices_to_remove = vec!();

    if !render.devices.is_empty() {
        render
            .devices
            .iter_mut()
            .for_each(|(key, single_device)| {
                // topic name which is used to get from hash_map
                let device_tab_name = String::from(key);
                
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
                    
                // some work on existing device
                //
                // prepare devices to be deleted
                } else {
                    match single_device.status {
                        ui::Status::OnPause => {},
                        _ => {
                            single_device.verify_status(&render.app.config,
                                                        &mut devices_to_remove,
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

//
fn startup() -> Result<()> {
    let mut stdout = std::io::stdout();

    stdout.execute(EnterAlternateScreen)?;
    
    enable_raw_mode()?;

    Ok(())
}

//
fn shutdown() -> Result<()> {
    let mut stdout = std::io::stdout();

    stdout.execute(LeaveAlternateScreen)?;
    
    disable_raw_mode()?;
    
    Ok(())
}
