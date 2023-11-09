#[allow(unused)]
use crate::info;
#[allow(unused)]
use crate::warn;
#[allow(unused)]
use crate::error;

use crate::config;
use crate::config::Config;
use crate::config::DateTime;

use crate::PALLETE;

use crate::pallete;
use crate::pallete::ColorIndex;
use crate::pallete::ColorRGB;

use crate::mqtt::Payload;
use crate::mqtt::Array;
use crate::mqtt::Temperature;
use crate::mqtt::CommonLog;
use crate::mqtt::LEN;
use crate::mqtt::POW;

use crate::app::App;

use crate::run::Devices;
use crate::run::Device2Tab;

use crate::alarma::Alarma;

use std::collections::VecDeque;

use std::rc::Rc;

use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Direction::Horizontal;
use ratatui::layout::Direction::Vertical;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::layout::Alignment;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::symbols::Marker;
use ratatui::text::Span;
use ratatui::widgets::Axis;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Chart;
use ratatui::widgets::Dataset;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::Cell;
use ratatui::widgets::Row;
use ratatui::widgets::Tabs;
use ratatui::widgets::Table;

//use ratatui::Frame;
//pub type Frame<'a> = ratatui::Frame<'a, ratatui::backend::CrosstermBackend<std::io::Stderr>>;
pub type Frame<'a> = ratatui::Frame<'a>;

pub const DATA_CAPACITY: usize = 100;
pub const DATA_ALARMA_CAPACITY: usize = 3;

// 1000ms / 25ms = 40fps/Hz
// TOTALY BLOCKS LAPTOP !!! study more why as TUI was ok
// + also verify if my esp code is fast enough 40hz + mqtt and so on ..
//pub const UI_REFRESH_DELAY: u64 = 250;

const TEMPERATURE_ERROR_SLICE_VALUE: f32 = 99.0;
const TEMPERATURE_DEFAULT_VALUE: f32 = 126.0;
const TEMPERATURE_MAX: f32 = -55.0;
const TEMPERATURE_MIN: f32 = 125.0;
const TEMPERATURE_BOUNDARY_OFFSET: f32 = 5.0;
pub const TEMPERATURE_INDEX_STEP: f32 = 0.25; // todo!() verify sensor resolution

const COLOR_TAB_TEXT: Color = Color::Green;
const COLOR_TAB_TEXT_SELECTED: Color = Color::Yellow;
const COLOR_VALUES_BG_BOUNDARY_MIN: Color = Color::Magenta;
const COLOR_VALUES_BG_BOUNDARY_MAX: Color = Color::Red;
const COLOR_VALUES_TEXT: Color = Color::Gray;
const COLOR_BAR_TEXT: Color = Color::Cyan;
const COLOR_MAP_TABLE_TEXT: Color = Color::Cyan;
const COLOR_STATUS_TEXT: Color = Color::DarkGray;
const COLOR_STATUS_BG: Color = Color::Black;
const COLOR_STATUS_ON_RECEIVE: Color = Color::Green;
const COLOR_STATUS_ON_PAUSE: Color = Color::Red;
const COLOR_STATUS_UNKNOWN: Color = Color::LightMagenta;
const COLOR_STATUS_TO_REMOVE: Color = Color::LightCyan;
const COLOR_NONE_MAP: Color = Color::Cyan;
const COLOR_NONE_MAP_CANVAS: Color = Color::Green;
const COLOR_NONE_MAP_VALUES: Color = Color::Black;

const COLOR_BLOCK_BORDER: Color = Color::DarkGray;
const COLOR_BLOCK_TITLE: Color = Color::Green;

// try harder -> as this number can hide bar minimal color/value
const BAR_LEN: usize = PALLETE.len() / 19;

const STATUS_INIT: &str = "init";
const STATUS_ON_PAUSE: &str = "on pause";
const STATUS_RECEIVING: &str = "receiving";
const STATUS_UNKNOWN: &str = "unknown";
const STATUS_TO_REMOVE: &str = "to_remove";

enum SplitBy {
    Ratio,
    #[allow(unused)]
    Max(u16),
    #[allow(unused)]
    Min(u16),
}

type UiValue = f64;

pub struct Render {
    pub app: App,
    pub devices: Devices,
    pub dynamic_tabs: Vec<Device2Tab>,
    pub common_log: CommonLog,
}

impl Render {
    //
    pub fn remove_device(&mut self,
                         device: &String,
    ) {
        // remove app tab
        self
            .app
            .tabs
            .remove(device);
        
        // remove dynamic tab
        self
            .dynamic_tabs
            .retain(|t| !t.eq(
                &Device2Tab::Dynamic(
                    // todo(!) try harder
                    String::from(device)
                )
            ));
    }

    //
    pub fn insert_device(&mut self,
                         device: Device2Tab,
    ) {
        // dynamic tab
        self.dynamic_tabs.push(device.clone());

        // app tab
        self.app.push_title(
            device.get_tab()
        )
    }
    
    /* // cannot use while render.devices.iter_mut()
    //
    // no title but tab
    pub fn insert_title(&mut self,
                        title: String,
    ) {
        self.app.push_title(app::Tab {
            name: title,
            variant: app::TabVariant::Fixed,
        });
    }

    //
    pub fn insert_tab(&mut self,
                      tab: Device2Tab,
    ) {
        self.dynamic_tabs.push(tab);
    }
    */
    
    //
    pub fn new(app: App) -> Self {
        Self {
            app,
            devices: std::collections::HashMap::new(),
            dynamic_tabs: vec![],
            common_log: CommonLog::default(),
        }
    }
    
    //
    // here we need data for specific TOPIC
    //
    fn draw_dynamic_tab(&mut self,
                        frame: &mut Frame,
                        area: Rect,
                        index: usize,
    ) {
   
        // here we need to read index number and connect it with hashmap key
        let topic = match index {
            // todo!() - try harder maybe enum as in previous match
            //i @ 1.. => {
            i @ 2.. => {
                //match tabs.get(i - 1) {
                match self.dynamic_tabs.get(i - 2) {
                    Some(tab) => {
                        match tab {
                            Device2Tab::Dynamic(name) => name,
                            // ? verify
                            Device2Tab::Fixed(name) => name,
                        }
                    },
                    // todo(!) -> this will not render anything !!!
                    None => self.app.config.mqtt_topic_base,
                }
            },
            _ => self.app.config.mqtt_topic_base,
        };
        
        // via topic match
        if let Some(single_device) = self.devices.get(topic) {
            let color_index = pallete::index_color_pallete(
                single_device.boundary_max.value,
                single_device.boundary_min.value,
            );
            
            let pixel_array = single_device.pixels(&color_index);
            
            // --> top (graph + bar) + bottom (grid + map + log)
            let chunks = Layout::default()
                .direction(Vertical)
                .constraints([
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ])
                .split(area);
        
            // top --> left graph + right bar
            let top = Layout::default()
                .direction(Horizontal)
                .constraints([
                    Constraint::Percentage(94),
                    Constraint::Percentage(6),
                ])
                .split(chunks[0]);
            
            // bottom --> left (value) + right (color + log)
            let chunks_bottom = split_area(chunks[1],
                                           Horizontal,
                                           2,
                                           SplitBy::Ratio,
            );
            
            // bottom --> right --> color map + log
            let chunks_right = Layout::default()
                .direction(Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ].as_ref())
                .split(chunks_bottom[1]);
            
            // bottom --> left --> value --> LEN * LEN canvas
            let inner_left  = inner_block(Some("left: values"),
                                          chunks_bottom[0],
                                          frame,
            );
            let chunks_lines_left = split_area(inner_left,
                                               Vertical,
                                               LEN,
                                               // ok, but big space between rows
                                               //SplitBy::Ratio,
                                               SplitBy::Max(1),
            );
            // bottom --> right --> color_map --> LEN * LEN canvas
            let inner_right_map  = inner_block(Some("right: map"),
                                          chunks_right[0],
                                          frame,
            );
            let chunks_lines_right = split_area(inner_right_map,
                                                Vertical,
                                                LEN,
                                                //
                                                SplitBy::Ratio,
                                                // same as Min(1)
                                                //SplitBy::Max(1),
                                                // tester - but small
                                                //SplitBy::Min(1),
            );
            
            // max + min graph
            draw_chart(
                "Graph",
                single_device,
                COLOR_VALUES_BG_BOUNDARY_MAX,
                COLOR_VALUES_BG_BOUNDARY_MIN,
                frame,
                top[0]
            );
            
            // heat bar
            draw_bar_as_tab(&self.app.config,
                            &color_index,
                            top[1],
                            frame, 
            );
        
            // grid + heatmap via single iter
            draw_map_and_values(&self.app.config,
                                chunks_lines_left,
                                chunks_lines_right,
                                pixel_array,
                                frame,
            );
            
            // logs
            let alarma = single_device.logs();
            
            draw_logs(
                &alarma,
                COLOR_STATUS_TEXT,
                COLOR_STATUS_BG,
                frame,
                chunks_right[1],
            );
        }
    }
    
    // error log
    //
    fn draw_error_log_topic(&mut self,
                            frame: &mut Frame,
                            area: Rect,
                            color_text: Color,
                            color_area: Color,
    ) {
        let items =
            self.common_log.logs
            .iter()
            .rev()
            .map(|log| ListItem::new(format!("{}", log))
                 .style(
                     Style::default()
                         .fg(color_text)
                         .bg(color_area)
                 )
            )
            .collect::<Vec<_>>();
        
        let list = List::new(items)
            .block(Block::default()
                   .title("Error_Logs")
                   .borders(Borders::ALL)
            );
        
        frame.render_widget(list, area);
    }
    
    // all heatmaps in one tab
    //
    // for now the upper limit is 3 sensors/heatmaps
    //
    // first we need to divide space
    // then collect data for each window
    // render
    // each topic has it's own temperature range!!!
    //
    fn draw_tab_heatmap_all(&mut self,
                            frame: &mut Frame,
                            area: Rect,
    ) {
        let chunks = split_area(area,
                                Direction::Horizontal,
                                self.devices.len(),
                                SplitBy::Ratio,
        );
        
        let mut device_counter = 0;
        
        self.devices
            .iter()
            .for_each(|(name, single_device)| {
                let chunks_inner = Layout::default()
                    .direction(Vertical)
                    .constraints([
                        //heatmap as table
                        Constraint::Percentage(20),
                        //heatmap via canvas
                        Constraint::Percentage(60),
                        //logs
                        Constraint::Percentage(20),
                    ])
                    // ### but here we ask for index 3, but we have only [0,1,2]
                    //.split(chunks[device_counter]);
                    // /*
                    .split(
                        // this can overflow
                        //chunks[device_counter]
                        // todo!() -> hot_fix
                        if chunks.len() <= device_counter {
                            chunks[0] //todo!() -> test such cases
                        } else {
                            chunks[device_counter]
                        }
                    );
                    // */
                
                let color_index = pallete::index_color_pallete(
                    single_device.boundary_max.value,
                    single_device.boundary_min.value,
                );
                
                let pixel_array = single_device.pixels(&color_index);
                
                // top map_as table
                // todo!() -> watch chunk index as this can panic/crash
                draw_map_as_table(&self.app.config,
                                  Some(&format!("map_as_table: {name}")),
                                  chunks_inner[0], 
                                  pixel_array.clone(),
                                  frame,
                );
                
                // heatmap_only
                draw_map_only(Some(&format!("map_only: {name}")),
                              chunks_inner[1],
                              pixel_array.clone(),
                              frame,
                );
                
                // logs
                let alarma = single_device.logs();
                
                draw_logs(
                    &alarma,
                    COLOR_STATUS_TEXT,
                    COLOR_STATUS_BG,
                    frame,
                    chunks_inner[2],
                );
                
                device_counter += 1;
            });
    }
    
    // learning sample + tab with fixed data --> for instance "colors"
    //
    fn draw_tab_fixed(&mut self,
                      frame: &mut Frame,
                      area: Rect,
    ) {
        // divide into left + right
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 2),
                Constraint::Ratio(1, 2),
            ])
            .split(area);
        
        let colors = [
            Color::Reset,
            Color::Black,
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::Gray,
            Color::DarkGray,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::LightCyan,
            Color::White,
        ];
        
        let items: Vec<Row> = colors
            .iter()
            .map(|c| {
                let cells = vec![
                    Cell::from(Span::raw(format!("{:?}: ", c))),
                    Cell::from(Span::styled("Foreground", Style::default().fg(*c))),
                    Cell::from(Span::styled("Background", Style::default().bg(*c))),
                ];
                
                Row::new(cells)
            })
            .collect();
        
        let table = Table::new(items)
            .block(Block::default()
                   .title("Colors")
                   .borders(Borders::ALL)
            )
            .widths(&[
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ]);
        
        frame.render_widget(table, chunks[0]);
    }
    
    //
    pub fn draw(&mut self,
                frame: &mut Frame,
    ) {
        // main window --> top tabs + bottom rest
        let chunks = Layout::default()
            .direction(Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0)
            ].as_ref())
            .split(frame.size());

        let tab_titles = self.app
            .tabs
            .titles
            .iter()
            .map(|t| Span::styled(t.render(),
                                  Style::default().fg(COLOR_TAB_TEXT)))
            .collect();
        
        let tabs = Tabs::new(tab_titles)
            .block(Block::default()
                   .title(self.app.config.app_title)
                   .borders(Borders::ALL)
            )
            .highlight_style(Style::default().fg(COLOR_TAB_TEXT_SELECTED))
            .select(self.app.tabs.index);
        
        // draw tabs
        frame.render_widget(tabs, chunks[0]);
        
        // draw rest under
        match self.app.tabs.index {
            // static -> table with colors
            0 => self.draw_tab_fixed(frame, chunks[1]),
            // static -> all heatmap side by side
            1 => self.draw_tab_heatmap_all(frame,
                                           chunks[1],
            ),
            // fixed -> error_log
            2 => self.draw_error_log_topic(frame,
                                           chunks[1],
                                           COLOR_STATUS_TEXT,
                                           COLOR_STATUS_BG,
            ),
            // dynamic tabs as topics
            //
            // todo!() -> maybe change to enum also as index can move due to fixed!!!
            index @ 3.. => self.draw_dynamic_tab(frame,
                                                 chunks[1],
                                                 index,
            ),
            // rest
            _ => {}
        };
    }
}

enum ShowIndex {
    True,
    False,
}

enum ShowColor {
    True,
    False,
}

#[derive(Clone, Copy)]
pub enum BoundaryVariant {
    Max,
    Min,
}

#[derive(Debug)]
pub enum Status {
    Init,
    OnPause,
    Receiving,
    Unknown,
    ToRemove,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "status: {}",
               match self {
                   Self::Init => {STATUS_INIT},
                   Self::OnPause => {STATUS_ON_PAUSE},
                   Self::Receiving => {STATUS_RECEIVING},
                   Self::Unknown => {STATUS_UNKNOWN},
                   Self::ToRemove => {STATUS_TO_REMOVE},
               },
        )
    }
}

pub struct Boundary {
    pub history: VecDeque<Temperature>,
    pub index: usize,
    pub value: Temperature
}

impl Boundary {
    pub fn new(variant: BoundaryVariant) -> Self {
        Self {
            history: VecDeque::default(),
            index: 0,
            value: match variant {
                BoundaryVariant::Max => { TEMPERATURE_MAX },
                BoundaryVariant::Min => { TEMPERATURE_MIN },
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Pixel {
    index: UiValue,
    value: UiValue,
    boundary_variant: Option<BoundaryVariant>,
    color: Option<ColorRGB>,
}

pub type Topic = String;
pub type Uuid = String;

// measurement data for rendering
pub struct Device {
    pub status: Status,
    // incomming topic
    // /grid_eye/queen/a4d1d8c1bc884b4abbbcf2b7a39a235a
    // 
    // /grid_eye/queen
    pub topic: Topic,
    // a4d1d8c1bc884b4abbbcf2b7a39a235a
    pub uuid: Uuid,
    pub array: Array,
    pub boundary_max: Boundary,
    pub boundary_min: Boundary,
    pub diff: Temperature,
    pub datetime_init: DateTime,
    pub datetime_last: DateTime,
    pub alarma: Option<Alarma>,
    pub alarma_history: VecDeque<Alarma>,
}

impl Device {
    //
    pub fn is_active(&mut self,
                     limit: chrono::Duration,
    ) -> bool {
        (config::now() - self.datetime_last) < limit
    }
    
    //
    // do not forget machine and simulator difference in topic !!!
    //
    pub fn verify_status(&mut self,
                         config: &Config,
                         devices_to_remove: &mut Vec<String>,
    ) {
        // todo(!) -> try harder / enum ???
        if !self.is_active(config.duration_limit_device_remove()) {
            devices_to_remove.push(self.topic.clone());
        } else if !self.is_active(config.duration_limit_device_remove_warn()) {
            self.status = Status::ToRemove;
        } else if !self.is_active(config.duration_limit_device_unknown()) {
            self.status = Status::Unknown;
        }
    }

    //
    pub fn init(config: &Config) -> Self {
        let now = config::now();

        Self {
            status: Status::Init,
            topic: String::from(config.default_empty_topic),
            uuid: String::from(config.default_empty_uuid),
            array: [TEMPERATURE_DEFAULT_VALUE; POW],
            boundary_max: Boundary::new(BoundaryVariant::Max),
            boundary_min: Boundary::new(BoundaryVariant::Min),
            diff: 0 as Temperature,
            datetime_init: now,
            datetime_last: now,
            alarma: None,
            alarma_history: VecDeque::default(), 
        }
    }

    //
    pub fn truncate(&mut self) {
        // historic data for min/max running graph
        self.boundary_min.history.truncate(DATA_CAPACITY);
        self.boundary_max.history.truncate(DATA_CAPACITY);
        // logs len
        self.alarma_history.truncate(DATA_ALARMA_CAPACITY);
    }

    //
    // Payload -> Data
    //
    pub fn fill(&mut self,
                config: &Config,
                channel_data: Payload,
                topic: Topic,
                uuid: Uuid,
    ) {
        self.status = Status::Receiving;

        // if sensor machine has booted we have new uuid
        if !self.uuid.eq(&uuid) {
            self.status = Status::Init;
            self.datetime_init = channel_data.datetime;
        }

        self.datetime_last = channel_data.datetime;
        
        self.topic = topic;
        self.uuid = uuid;
        
        self.array = channel_data.array;

        self.boundary_min.history.push_front(channel_data.min.value);
        self.boundary_min.index = channel_data.min.index;
        self.boundary_min.value = channel_data.min.value;

        self.boundary_max.history.push_front(channel_data.max.value);
        self.boundary_max.index = channel_data.max.index;
        self.boundary_max.value = channel_data.max.value;
        
        self.diff = channel_data.max.value - channel_data.min.value;

        self.alarma(config);
    }

    //
    fn pixels(&self,
              color_index: &pallete::ColorIndex,
    ) -> Vec<Pixel> {
        self
            .array
            .iter()
            .enumerate()
            .map(|(index, value): (usize, &Temperature)| {
                let boundary_variant = if index.eq(&self.boundary_min.index) { Some(BoundaryVariant::Min)
                } else if index.eq(&self.boundary_max.index) { Some(BoundaryVariant::Max)
                } else { None };
                
                Pixel {
                    index: index as UiValue,
                    value: *value as UiValue,
                    boundary_variant,
                    color: pallete::temperature_to_color(color_index.as_slice(),
                                                         *value as Temperature,
                    ),
                }
            })
            .collect::<Vec<_>>()
    }

    //
    fn logs(&self) -> Vec<String> {
        let logs_topic = format!("topic: {}", self.topic);
        let logs_uuid = format!("uuid: {}", self.uuid);

        let logs_init = format!("init: {}", self.datetime_init);
        let logs_last = format!("last: {}", self.datetime_last);

        let uptime = self.datetime_last - self.datetime_init;
        let logs_uptime_minutes =
            format!("uptime: {:?} minutes",
                    uptime.num_minutes(),
            );

        let alarma = match &self.alarma {
            Some(a) => {
                format!("alarma: {:02.02} - {:02.02} -> {:02.02}",
                        a.max,
                        a.min,
                        a.diff,
                )
            },
            None => {
                String::from("alarma: None")
            },
        };
       
        let mut logs: Vec<String> = vec![
            logs_topic,
            logs_uuid,
            self.status.to_string(),
            logs_init,
            logs_last,
            logs_uptime_minutes,
            alarma,
        ];

        let alarma_history = self.alarma_history
            .iter()
            .map(|a| {
                // why i do not get leading zeroes for diff ???
                format!(" {:?} / {:02.02}",
                        a.datetime,
                        a.diff,
                )
            })
            .collect::<Vec<String>>();

        logs.extend(alarma_history);

        logs
    }

    // actual alarma
    //
    pub fn alarma(&mut self,
                  config: &Config,
    ) {
        if self.diff >= config.alarma_diff {
            let alarma = Alarma {
                max: self.boundary_max.value,
                min: self.boundary_min.value,
                diff: self.diff, 
                datetime: self.datetime_last, 
            };
            
            self.alarma = Some(alarma.clone());
            self.alarma_history.push_front(alarma);
        } else {
            self.alarma = None;
        }
    }
} 

//
fn draw_chart(
    title: &str,
    device: &Device,
    color_graph_max: Color,
    color_graph_min: Color,
    frame: &mut Frame,
    area: Rect,
) {
    let info_max = format!("temperature_max: {:02.02}", device.boundary_max.value);
    let info_min = format!("temperature_min: {:02.02}", device.boundary_min.value);

    let mut boundary_max: Temperature = TEMPERATURE_MAX;
    let mut boundary_min: Temperature = TEMPERATURE_MIN;
    
    let device_max = data_history_format(&device.boundary_max.history,
                                       &mut boundary_max,
                                       Some(BoundaryVariant::Max),
    );
    
    let device_min = data_history_format(&device.boundary_min.history,
                                       &mut boundary_min,
                                       Some(BoundaryVariant::Min),
    );
    
    let dataset = vec![
        Dataset::default()
            .name(&info_max)
            .marker(Marker::Dot)
            .style(Style::default().fg(color_graph_max))
            .graph_type(ratatui::widgets::GraphType::Scatter)
            .data(device_max.as_slice()),
        Dataset::default()
            .name(&info_min)
            .marker(Marker::Braille)
            .style(Style::default().fg(color_graph_min))
            .graph_type(ratatui::widgets::GraphType::Line)
            .data(device_min.as_slice()),
    ];

    let boundary_min_with_offset = (boundary_min.floor() - TEMPERATURE_BOUNDARY_OFFSET) as UiValue;
    let boundary_max_with_offset = (boundary_max.ceil() + TEMPERATURE_BOUNDARY_OFFSET) as UiValue;
    
    let chart = Chart::new(dataset)
        .block(Block::default()
               .title(title)
               .borders(Borders::ALL)
        )
        .x_axis(
            Axis::default()
                // text color
                .title(Span::styled(
                    format!("X Axis | diff: {:02.02}", device.diff),
                    Style::default().fg(Color::Green)),
                )
                // border color
                .style(Style::default().fg(Color::White))
                .bounds([0.0, DATA_CAPACITY as UiValue]),
        )
        .y_axis(
            Axis::default()  
                .title(Span::styled("Y Axis", Style::default().fg(Color::Green)))
                .style(Style::default().fg(Color::White))
                .bounds([
                    boundary_min_with_offset,
                    boundary_max_with_offset,
                        
                ])
                .labels(vec![
                    Span::raw(format!("{:02.02}", boundary_min_with_offset)),
                    Span::raw(format!("{:02.02}", boundary_max_with_offset)),
                ]),
        );

    frame.render_widget(chart, area);
}

// draw bar as table with single cell per row
//
fn draw_bar_as_tab(config: &Config,
                   color_index: &ColorIndex,
                   chunks: Rect,
                   frame: &mut Frame,
) {
    let items: Vec<Row> =
        color_index
        .iter()
        .rev()
        .step_by(BAR_LEN)
        .enumerate()
        .map(|(index, color)| {
            let (red, green, blue) = color.1;

            let cells = vec![
                Cell::from(
                    Span::styled(
                        if config.flag_show_bar_index.eq(&true) {
                            format!("{:02}|{:02.02}",
                                    index,
                                    color.0,
                            )
                        } else { format!("{:02.02}", color.0) },
                        Style::default()
                            .fg(COLOR_BAR_TEXT)
                            /*
                            .bg(match color.1 {
                                (red, green, blue) => Color::Rgb(red, green, blue),
                            })
                            */
                            .bg(Color::Rgb(red, green, blue))
                    ),
                )
            ];
            
            Row::new(cells).height(1)
        }).collect();

    let table = Table::new(items)
        .block(Block::default()
               .title("Bar")
               .borders(Borders::ALL)
        )
        //.header(Row::new(vec!["Celsius"]))
        .widths(&[Constraint::Ratio(1, 1)])
        //.widths([Constraint::Length(25), Constraint::Min(0)].as_ref())
        //.column_spacing(1)
        //.style(Style::default().fg(Color::Green))
        ;
        
    frame.render_widget(table, chunks);
}

//
fn draw_map_as_table(config: &Config,
                     title: Option<&str>,
                     chunks: Rect,
                     array: Vec<Pixel>,
                     frame: &mut Frame,
) {

    let lines: Vec<Row> = (0..LEN)
        .map(|row| {
            let cells = (0..LEN)
                .map(|cell| {
                    let index = (row * LEN) + cell;
                    let pixel = get_pixel(&array,
                                          row,
                                          cell,
                    );

                    build_cell(config,
                               pixel,
                               index,
                    )
                }).collect::<Vec<Cell>>();
            
            Row::new(cells)
                //.height(2) // it add's blank/black space, not enlarge bg
                //.bottom_margin(1) // 1 set space in between
        }).collect();

    let rows_count = lines.len() as u32;
    let mut rows = (0..rows_count)
        .map(|_row_index|
             // we have space between cells, even with added Constraint::Min(0)
             //Constraint::Ratio(1, rows_count)
             // ok, nice and without space between cells
             Constraint::Min(5) // 5 as we have {:02.02}
             // add's space blank/black where there is no char
             //Constraint::Min(8)
             // tester
             //Constraint::Max(5)
        )
        .collect::<Vec<Constraint>>();

    rows.push(Constraint::Min(0));
    
    let block = match title {
        Some(t) => Block::default().title(t),
        None => Block::default(),
    }.borders(Borders::ALL);
    
    let table = Table::new(lines)
        .block(block)
        /*
        .header(
            Row::new(vec!["0", "1", "2"," 3", "4", "5", "6", "7"])
        )
        */
        .widths(&rows)
        .column_spacing(0); // default is 1

frame.render_widget(table, chunks);
}
    
// map render: value + color
// two chunks as via single iter
//
fn draw_map_and_values(config: &Config,
                       chunks_lines_left: Rc<[Rect]>,
                       chunks_lines_right: Rc<[Rect]>,
                       array: Vec<Pixel>,
                       frame: &mut Frame,
) {
    /*
    // add block around left and right
    let inner_left  = inner_block(Some("left values"),
                                  chunks_lines_left,
                                  frame,
    );
    */
    
    // todo(!) --> measure duration + async
    (0..LEN)
        .for_each(|row| {
            // todo(!) --> this two can go async
            let chunks_cell_left = split_area(chunks_lines_left[row], // index not safe!!
                                              Horizontal,
                                              LEN,
                                              // ok, but blank/black space
                                              //SplitBy::Ratio,
                                              // ok, but need to add +1 
                                              // 8 as {:02}|{:02.02}
                                              SplitBy::Min(8+1),
            );

            let chunks_cell_right = split_area(chunks_lines_right[row],
                                               Horizontal,
                                               LEN,
                                               //
                                               SplitBy::Ratio,
                                               // ok, but small
                                               // 4 - laptop miss last col
                                               //SplitBy::Max(3),
                                               // tester
                                               //SplitBy::Min(3),
            );

            // todo(!) --> try rayon for first time ???
            (0..LEN)
                .for_each(|cell| {
                    let pixel = get_pixel(&array,
                                          row,
                                          cell,
                    );
                    
                    // LEFT <- TEMPERATURE
                    if let Some(ch) = chunks_cell_left.get(cell) {
                        // CANVAS_VALUES
                        show_canvas_values(
                            pixel,
                            COLOR_VALUES_TEXT, // text
                            pixel.color, // bg
                            if config.flag_show_index.eq(&true) { ShowIndex::True } else { ShowIndex::False },
                            if config.flag_show_color.eq(&true) { ShowColor::True } else { ShowColor::False },
                            frame,
                            *ch,
                        );
                    }
                    
                    // RIGHT -> COLOR MAP
                    if let Some(ch) = chunks_cell_right.get(cell) {
                        // CANVAS_COLOR
                        show_canvas_color(
                            pixel.color,
                            frame,
                            *ch,
                        );
                    }
                })
        });
}

// map render: color
//
// used in all_heatmap for each device
//
fn draw_map_only(title: Option<&str>,
                 chunk: Rect,
                 array: Vec<Pixel>,
                 frame: &mut Frame,
) {
    let inner = inner_block(title,
                            chunk,
                            frame,
    );

    let chunks_lines = split_area(inner, //chunks,
                                  Vertical,
                                  LEN,
                                  // ok, full but not rectangle
                                  SplitBy::Ratio,
                                  // ok, but small at big screen
                                  //SplitBy::Max(2),
    );

    // todo(!) --> measure duration + async
    (0..LEN)
        .for_each(|row| {
            let chunks_cells = split_area(chunks_lines[row],
                                          Horizontal,
                                          LEN,
                                          // ok, full but not rectangle
                                          SplitBy::Ratio,
                                          // ok, but small at big screen
                                          //SplitBy::Max(2*2),
            );

            // todo(!) --> try rayon for first time ???
            (0..LEN)
                .for_each(|cell| {
                    let pixel = get_pixel(&array,
                                          row,
                                          cell,
                    );
                    
                    if let Some(ch) = chunks_cells.get(cell) {
                        show_canvas_color(
                            pixel.color,
                            frame,
                            *ch,
                        );
                    }
                })
        });
}

// display logs
//
fn draw_logs(
    logs: &[String],
    color_text: Color,
    color_area: Color,
    frame: &mut Frame,
    area: Rect,
) {
    let items =
        logs
        .iter()
        .map(|log| ListItem::new(log.to_string())
             .style(
                 Style::default()
                     .fg(
                         // try harder -> just quick info if paused !!!
                         // pause will delete and later not add new !!!
                         if log.contains(STATUS_ON_PAUSE) {
                             COLOR_STATUS_ON_PAUSE
                         } else if log.contains(STATUS_RECEIVING) {
                             COLOR_STATUS_ON_RECEIVE
                         } else if log.contains(STATUS_UNKNOWN) {
                             COLOR_STATUS_UNKNOWN
                         } else if log.contains(STATUS_TO_REMOVE) {
                             COLOR_STATUS_TO_REMOVE
                         } else {
                             color_text
                         }
                     )
                     .bg(color_area)
             )
        )
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(Block::default()
               .title("Logs")
               .borders(Borders::ALL)
        );
    
    frame.render_widget(list, area);
}

// display single pixel as value
// min/max values are highlighted with color
//
fn show_canvas_values(
    pixel: Pixel,
    color_text: Color,
    color_area: Option<ColorRGB>,
    show_index: ShowIndex,
    show_color: ShowColor,
    frame: &mut Frame,
    area: Rect,
) {
    let canvas = Canvas::default()
        .block(Block::default())
        .background_color(
            match show_color {
                ShowColor::True => match color_area {
                    Some((red, green, blue)) => Color::Rgb(red, green, blue),
                    None => COLOR_NONE_MAP_VALUES,
                },
                ShowColor::False => {
                    match pixel.boundary_variant {
                        Some(boundary) => match boundary {
                            BoundaryVariant::Max => COLOR_VALUES_BG_BOUNDARY_MAX,
                            BoundaryVariant::Min => COLOR_VALUES_BG_BOUNDARY_MIN,
                        },
                        None => COLOR_NONE_MAP_VALUES,
                    }
                }
            }
        )
        .paint(|ctx| {
            ctx.print(
                0 as UiValue,
                0 as UiValue,
                Span::styled(
                    match show_index {
                        ShowIndex::True => format!("{:02}|{:02.02}", pixel.index, pixel.value),
                        ShowIndex::False => format!("{:02.02}", pixel.value)
                    },
                    Style::default().fg(color_text),
                ),
            )
        });
    
    frame.render_widget(canvas, area);
}

// display single pixel as color
//
// todo!() -> is there a way to define it's size ???
//
fn show_canvas_color(
    color_area: Option<ColorRGB>,
    frame: &mut Frame,
    area: Rect,
) {

    let canvas = Canvas::default()
        .block(Block::default())
        .background_color( match color_area {
            Some((red, green, blue)) => Color::Rgb(red, green, blue),
            None => COLOR_NONE_MAP_CANVAS,
        })
        .paint(|_ctx| {} );
    
    frame.render_widget(canvas, area);
}

//
fn split_area(input: Rect,
              direction: Direction,
              size: usize,
              split: SplitBy,
) -> Rc<[Rect]> {
    let mut v = (0..size)
        .map(|_|
             match split {
                 SplitBy::Ratio => Constraint::Ratio(1, size as u32),
                 SplitBy::Min(v) => Constraint::Min(v),
                 SplitBy::Max(v) => Constraint::Max(v),
             }
        )
        .collect::<Vec<_>>();

    // last chunk will be not expanded
    // this work's ok only for SplitBy::Min and ::Max
    v.push(Constraint::Min(0));
    
    Layout::default()
        .direction(direction)
        .constraints(v.as_ref())
        .split(input)
}

// index data for ui chart display
// find boundary value in capacity_data
//
fn data_history_format(data: &VecDeque<Temperature>,
                       boundary: &mut Temperature,
                       side: Option<BoundaryVariant>,
) -> Vec<(UiValue, UiValue)> {
    data
        .iter()
        .rev()
        .enumerate()
        .map(|(index, value): (usize, &Temperature)| {
            if let Some(variant) = side {
                match variant {
                    BoundaryVariant::Max => if *boundary < *value { *boundary = *value },
                    BoundaryVariant::Min => if *boundary > *value { *boundary = *value },
                }
            }
            
            (index as UiValue, *value as UiValue)
        })
        .collect::<Vec<_>>()
}

//
fn get_pixel(array: &Vec<Pixel>,
             row: usize,
             cell: usize,
) -> Pixel {
    let index = (row * LEN) + cell;

    match array.as_slice().get(index) {
        Some(r) => *r,
        None => Pixel {
            index: index as UiValue,
            value: TEMPERATURE_ERROR_SLICE_VALUE as UiValue,
            boundary_variant: None,
            color: None,
        },
    }
}

//
fn build_cell(config: &Config,
              pixel: Pixel,
              index: usize) -> Cell<'static> {
    Cell::from(
        Span::styled(
            // /* // TEXT + STYLE AS TABLE DATA
            // /* // temperature as text 
            /* // table need some text to have size !!!
            format!("  "),
             */
            if config.flag_show_map_table_index.eq(&true) {
                format!("{:02}|{:02.02}",
                        index,
                        pixel.value,
                )
            } else { format!("{:02.02}", pixel.value) },
            Style::default()
                .fg(COLOR_MAP_TABLE_TEXT)
                .bg(match pixel.color {
                    Some((red, green, blue)) => Color::Rgb(red, green, blue),
                    None => COLOR_NONE_MAP,
                })
        )
    )
}

//
// especially for heatmap(lot's of canvas rectangles)
// we want to have it inside block
// as we can add border and title
//
fn inner_block(title: Option<&str>,
               chunk: Rect,
               frame: &mut Frame,
) -> Rect {
    let block = match title {
        Some(t) => {
            Block::default()
                .title(t)
                .title_style(Style::default().fg(COLOR_BLOCK_TITLE))
                .title_alignment(Alignment::Center)
        },
        None => {
            Block::default()
        }
    };

    let block = block
        .style(Style::default())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_BLOCK_BORDER));
    
    let inner = block.inner(chunk);
    frame.render_widget(block, chunk);
                        
    inner
}
