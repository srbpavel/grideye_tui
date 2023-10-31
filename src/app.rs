#[allow(unused)]
use crate::info;
#[allow(unused)]
use crate::warn;
#[allow(unused)]
use crate::error;

use crate::config::Config;

const TABVARIANT_STATIC: &str = "STATIC";
const TABVARIANT_FIXED: &str = "FIXED";
const TABVARIANT_DYNAMIC: &str = "DYNAMIC";

#[derive(Debug)]
pub enum TabVariant {
    Static,
    Fixed,
    Dynamic,
}

impl std::fmt::Display for TabVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "{}",
               match self {
                   Self::Static => {TABVARIANT_STATIC},
                   Self::Fixed => {TABVARIANT_FIXED},
                   Self::Dynamic => {TABVARIANT_DYNAMIC},
               },
        )
    }
}

#[derive(Debug)]
pub struct Tab {
    pub name: String,
    pub variant: TabVariant,
} 

impl Tab {
    //
    pub fn new(name: String,
               variant: TabVariant) -> Self {
        Self {
            name,
            variant,
        }
    }

    //
    pub fn render(&self) -> String {
        format!("{}: {}",
                self.variant,
                self.name,
        )
    }
}

#[derive(Debug)]
pub struct TabsState {
    pub titles: Vec<Tab>,
    pub index: usize,
}

impl TabsState {
    //
    pub fn new(titles: Vec<Tab>) -> Self {
        Self {
            titles,
            index: 0,
        }
    }

    //
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    //
    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }

    // remove inactive device tab
    pub fn remove(&mut self,
                  device: &String,
    ) {
        self.titles.retain(|d| !d.name.eq(device));
    }
}

pub struct App {
    pub config: Config,
    pub should_quit: bool,
    pub should_pause: bool,
    pub tabs: TabsState,
}

impl App {
    //
    pub fn new(config: Config) -> Self {
        Self {
            config,
            should_quit: false,
            should_pause: false,
            tabs: TabsState::new(vec![]),
        }
    }

    // todo!
    //pub fn tick(&self) {}
    
    pub fn on_right(&mut self) {
        self.tabs.next();
    }
    
    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_pause(&mut self) {
        self.should_pause = !self.should_pause.eq(&true);
    }
    
    pub fn on_key(&mut self, c: char) {
        match c {
            // quit
            'q' => {
                self.should_quit = true;
            },
            // pause
            'p' => {
                self.on_pause();
            },
            // spacebar
            ' ' => {
                self.on_right();
            },
            // tabs via number
            ascii_i @ '1'..='9' => {
                /*
                // ascii table Dec value for Char 0 is 48
                //let index = ascii_i as usize - 48;
                if index > self.tabs.titles.len() {
                     self.tabs.index = 0;
                 } else {
                     self.tabs.index = index - 1;
                }
                */

                if let Some(index) = ascii_i.to_digit(10) {
                    let index = index as usize;

                    if index > self.tabs.titles.len() {
                        self.tabs.index = 0;
                    } else {
                        self.tabs.index = index - 1;
                    }
                }
            },
            _ => {},
        }
    }

    //
    pub fn on_ctrl_key(&mut self, c: char) {
        match c {
            'c' | 'x' => {
                self.should_quit = true;
            }
            'f' => {
                self.on_right();
            }
            'b' => {
                self.on_left();
            }
            _ => {},
        }
    }

    //
    pub fn push_title(&mut self,
                      tab: Tab,
    ) {
        self
            .tabs
            .titles
            .push(tab)
    }
}
