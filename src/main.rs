mod pallete;
//use pallete::PALLETE_RYGB_SORTED as PALLETE;
//use pallete::IRON_BOW as PALLETE;
use pallete::IRON_BOW_LONG as PALLETE;

use std::thread;
use std::io;

use std::time::Duration;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

//use std::time::Instant;

use uuid::Uuid;

use rumqttc::Client;
use rumqttc::MqttOptions;
use rumqttc::Packet;
use rumqttc::QoS;

use tui::Terminal;
use tui::Frame;
use tui::backend::Backend;

use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Direction::Horizontal;
use tui::layout::Direction::Vertical;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Style;
use tui::symbols::Marker;
use tui::text::Span;
use tui::widgets::Axis;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Chart;
use tui::widgets::Dataset;
use tui::widgets::canvas::Canvas;

use tui::widgets::List;
use tui::widgets::ListItem;
use tui::style::Modifier;

// CROSSTERM
use tui::backend::CrosstermBackend;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::execute;
/* // key press
use crossterm::event::read;
use crossterm::event::KeyCode;
use crossterm::event::Event;
*/
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;

// 1000ms / 25ms = 40fps/Hz
const UI_REFRESH_DELAY: Duration = Duration::from_millis(25);
const DATA_CAPACITY: usize = 100;

const COLOR_NONE_MAP: Color = Color::Black;
const COLOR_NONE_BAR: Color = Color::Magenta;

const FLAG_SHOW_INDEX: bool = false;
const FLAG_SHOW_BAR_INDEX: bool = false;
const FLAG_SHOW_COLOR: bool = false;

const TEMPERATURE_ERROR_VALUE: f32 = 86.0;
const TEMPERATURE_ERROR_SLICE_VALUE: f32 = 99.0;
const TEMPERATURE_DEFAULT_VALUE: f32 = 126.0;
const TEMPERATURE_MAX: f32 = -55.0;
const TEMPERATURE_MIN: f32 = 125.0;
const TEMPERATURE_BOUNDARY_OFFSET: f32 = 5.0;
const TEMPERATURE_INDEX_STEP: f32 = 0.25; // todo!() verify sensor resolution

const LEN: usize = 8;
const POW: usize = LEN * LEN;
const CHUNK_SIZE: usize = 4;
const MQTT_PAYLOAD_SIZE: usize = POW * CHUNK_SIZE;

const MQTT_HOST: &str = "192.168.0.103";
const MQTT_USER: &str = "";
const MQTT_PASS: &str = "";
const MQTT_PORT: u16 = 1883;
const MQTT_TOPIC: &str = "/grid_eye/";
const MQTT_CLIENT_ID: &str = "grideye_tui";

enum ShowIndex {
    True,
    False,
}

enum ShowColor {
    True,
    False,
}

#[derive(Clone, Copy)]
enum BoundaryVariant {
    Max,
    Min,
}

type Temperature = f32;
type UiValue = f64;
type ColorIndex = Vec<(Temperature, (u8, u8, u8))>;
type Array =  [Temperature; POW];

struct ChannelData {
    min_value: Temperature,
    min_index: usize,
    max_value: Temperature,
    max_index: usize,
    array: Array,
}

#[derive(Clone, Copy)]
struct Pixel {
    index: UiValue,
    value: UiValue,
    boundary_variant: Option<BoundaryVariant>,
    color: Option<(u8, u8, u8)>,
}

struct Boundary {
    history: VecDeque<Temperature>,
    index: usize,
    value: Temperature
}

impl Boundary {
    fn new(variant: BoundaryVariant) -> Self {
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

struct Data {
    capacity: usize,
    array: Array,
    boundary_max: Boundary,
    boundary_min: Boundary,
    diff: Temperature,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            capacity: DATA_CAPACITY,
            array: [TEMPERATURE_DEFAULT_VALUE; POW],
            boundary_max: Boundary::new(BoundaryVariant::Max),
            boundary_min: Boundary::new(BoundaryVariant::Min),
            diff: 0 as Temperature,
        }
    }
}

impl Data {
    fn truncate(&mut self) {
        self.boundary_min.history.truncate(self.capacity);
        self.boundary_max.history.truncate(self.capacity);
    }
}

//
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // STOP SIGNAL
    let running = Arc::new(AtomicBool::new(true));
    let run_render_loop = running.clone();

    // CROSSTERM
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout,
             EnterAlternateScreen,
             EnableMouseCapture,
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // PREPARE TERMINAL
    terminal.clear().unwrap();
    terminal.hide_cursor().unwrap();
    
    // LAUNCH MEASUREMENT THREAD
    let mut data = Data::default();
    let (sender, receiver) = mpsc::channel();
    mqtt_payload_parse(sender);

    while run_render_loop.load(Ordering::SeqCst) {
        for channel_data in receiver.try_iter() {
            data.array = channel_data.array;

            // DEFAULT dynamic
            data.boundary_min.history.push_front(channel_data.min_value);
            data.boundary_min.index = channel_data.min_index;
            data.boundary_min.value = channel_data.min_value; // we have it in history also!!!

            data.boundary_max.history.push_front(channel_data.max_value);
            data.boundary_max.index = channel_data.max_index;
            data.boundary_max.value = channel_data.max_value;

            data.diff = channel_data.max_value - channel_data.min_value;
        }

        data.truncate();

        render(&mut terminal, &mut data);

        // todo! test via timer
        thread::sleep(UI_REFRESH_DELAY);
    }

    // restore terminal
    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    Ok(())
}

// terminal split to chunks + show data
fn render<B>(terminal: &mut Terminal<B>,
             data: &mut Data)
where
    B: Backend,
{
    
    let draw_status = terminal
        // &mut Frame<'_, B>  where B: Backend
        .draw(|frame| {
            // COLOR
            let color_index = index_color_pallete(data.boundary_max.value, data.boundary_min.value);

            // 20 * 25 = 500
            //let bar_len = PALLETE.len() / 20; // todo!() study more as cannot get more !!!
            // IRON_BOW
            //let bar_len = PALLETE.len() / 8; // todo!() study more as cannot get more !!!
            // LONG; 433 / 25 = 17.32
            let bar_len = PALLETE.len() / 17; // todo!() study more as cannot get more !!!
            
            // top_chart + bottom_map
            let chunks = Layout::default()
                .direction(Vertical)
                .constraints([
                    Constraint::Percentage(80),
                    Constraint::Percentage(20),
                ].as_ref())
                .split(frame.size());

            // top: chart + bar
            let chunks_top = Layout::default()
                .direction(Horizontal)
                .constraints([
                    Constraint::Percentage(97),
                    Constraint::Percentage(3),
                ].as_ref())
                .split(chunks[0]);

            // top_right: bar + bottom blank_space
            let chunks_bar = Layout::default()
                .direction(Vertical)
                .constraints([
                    //Constraint::Percentage(75),
                    //Constraint::Percentage(25),
                    // IRON_BOW
                    //Constraint::Percentage(47),
                    //Constraint::Percentage(53),
                    Constraint::Percentage(75),
                    Constraint::Percentage(25),
                ].as_ref())
                .split(chunks_top[1]);

            // many many color bars
            let chunks_top_right_bar = split_area(
                chunks_bar[0],
                Vertical,
                bar_len,
            );

            // bottom: left_value + right_color
            let chunks_bottom = split_area(chunks[1], Horizontal, 2);
            // bottom_right: color + blank_space
            let chunks_right = Layout::default()
                .direction(Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ].as_ref())
                .split(chunks_bottom[1]);

            // map_grid
            let chunks_lines_left = split_area(chunks_bottom[0], Vertical, LEN);
            // PERCENT
            let chunks_lines_right = split_area(chunks_right[0], Vertical, LEN);
            /* // LENGTH
            let chunks_lines_right = split_area_fixed(chunks_right[0],
                                                      Vertical,
                                                      LEN,
                                                      1_u16,
            );
            */
            
            let chunks_top_chart_info = format!(
                "FoOoKuMe is KiNg >>> chunks_bar: {}",
                chunks_top_right_bar.len(),
            );
            
            // ARRAY
            // measure duration + async
            //let start = Instant::now();
            let array = data
                .array
                .iter()
                .enumerate()
                .map(|(index, value): (usize, &Temperature)| {
                    let boundary_variant = if index.eq(&data.boundary_min.index) { Some(BoundaryVariant::Min)
                    } else if index.eq(&data.boundary_max.index) { Some(BoundaryVariant::Max)
                    } else { None };

                    Pixel {
                        index: index as UiValue,
                        value: *value as UiValue,
                        boundary_variant,
                        color: pallete::temperature_to_color(
                            color_index.as_slice(),
                            *value as Temperature,
                        ),
                    }
                })
                .collect::<Vec<_>>();
            //let stop = Instant::now();
            
            // CHART
            show_chart(
                &chunks_top_chart_info,
                data,
                Color::Red,
                Color::Cyan,
                frame,
                chunks_top[0],
            );

            // /*
            // BAR
            draw_bar(&color_index,
                     chunks_top_right_bar,
                     frame,
            );
            // */

            // MAP RENDER: values + color
            draw_map(chunks_lines_left,
                     chunks_lines_right,
                     array,
                     frame,
            );

            // /*
            // STATUS_CELL
            show_status(//data,
                ["a1", "b2", "c3", "d4", "e5"].as_slice(),
                Color::DarkGray,
                Color::Black,
                frame,
                chunks_right[1],
            );
            // */
        });

    // todo! send to log's
    match draw_status {
        Ok(_s) => {},
        Err(_e) => {},
    }
}

//
fn show_chart<B>(
    title: &str,
    data: &Data,
    color_graph_max: Color,
    color_graph_min: Color,
    frame: &mut Frame<B>,
    area: Rect,
)
where
    B: Backend
{
    let info_max = format!("temperature_max: {:02.02}", data.boundary_max.value);
    let info_min = format!("temperature_min: {:02.02}", data.boundary_min.value);

    let mut boundary_max: Temperature = TEMPERATURE_MAX;
    let mut boundary_min: Temperature = TEMPERATURE_MIN;
    
    let data_max = data_history_format(&data.boundary_max.history,
                                       &mut boundary_max,
                                       Some(BoundaryVariant::Max),
    );
    
    let data_min = data_history_format(&data.boundary_min.history,
                                       &mut boundary_min,
                                       Some(BoundaryVariant::Min),
    );
    
    let dataset = vec![
        Dataset::default()
            .name(&info_max)
            .marker(Marker::Dot)
            .style(Style::default().fg(color_graph_max))
            .graph_type(tui::widgets::GraphType::Scatter)
            .data(data_max.as_slice()),
        Dataset::default()
            .name(&info_min)
            .marker(Marker::Braille)
            .style(Style::default().fg(color_graph_min))
            .graph_type(tui::widgets::GraphType::Line)
            .data(data_min.as_slice()),
    ];

    let boundary_min_with_offset = (boundary_min.floor() - TEMPERATURE_BOUNDARY_OFFSET) as UiValue;
    let boundary_max_with_offset = (boundary_max.ceil() + TEMPERATURE_BOUNDARY_OFFSET) as UiValue;
    
    let chart = Chart::new(dataset)
        .block(Block::default().title(title).borders(Borders::ALL))
        .x_axis(
            Axis::default()
                // text color
                .title(Span::styled(
                    format!("X Axis | diff: {:02.02}", data.diff),
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

    // Frame::render_widget(impl Widget, Rect)
    frame.render_widget(chart, area);
}

// merge these two FN
// display single pixel as value
// min/max values are highlighted with color
//
fn show_canvas_values<B>(
    pixel: Pixel,
    color_text: Color,
    color_area: Option<(u8, u8, u8)>,
    show_index: ShowIndex,
    show_color: ShowColor,
    frame: &mut Frame<B>,
    area: Rect,
)
where
    B: Backend
{
    let canvas = Canvas::default()
        .block(Block::default())
        .background_color(
            match show_color {
                ShowColor::True => match color_area {
                    Some((red, green, blue)) => Color::Rgb(red, green, blue),
                    None => COLOR_NONE_MAP,
                },
                ShowColor::False => {
                    match pixel.boundary_variant {
                        Some(boundary) => match boundary {
                            BoundaryVariant::Max => Color::Red,
                            BoundaryVariant::Min => Color::DarkGray,
                        },
                        None => COLOR_NONE_MAP,
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
fn show_canvas_color<B>(
    color_area: Option<(u8, u8, u8)>,
    frame: &mut Frame<B>,
    area: Rect,
)
where
    B: Backend
{

    let canvas = Canvas::default()
        .block(Block::default())
        .background_color( match color_area {
            Some((red, green, blue)) => Color::Rgb(red, green, blue),
            None => COLOR_NONE_MAP,
        })
        .paint(|_ctx| {} );
    
    frame.render_widget(canvas, area);
}

//
#[allow(unused)]
fn show_status<B>(
    msg: &[&str],
    color_text: Color,
    color_area: Color,
    frame: &mut Frame<B>,
    area: Rect,
)
where
    B: Backend
{
    let items =
        msg
        .iter()
        .map(|msg| ListItem::new(msg.to_string())
             .style(Style::default()
                    .add_modifier(Modifier::BOLD)
             )
        )
        .collect::<Vec<_>>();

    let list = List::new(items)
        .block(Block::default()
               .title("List")
               .borders(Borders::ALL)
        )
        .style(Style::default()
               .fg(color_text)
               .bg(color_area)
        );
    
    frame.render_widget(list, area);
}

// display color pallette and it's values
//
fn show_canvas_bar<B>(
    symbol: String,
    color_text: Color,
    color_area: Option<(u8, u8, u8)>,
    frame: &mut Frame<B>,
    area: Rect,
)
where
    B: Backend
{
    let canvas = Canvas::default()
        .block(Block::default())
        .background_color( match color_area {
            Some((red, green, blue)) => Color::Rgb(red, green, blue),
            None => COLOR_NONE_BAR,
        })
        .paint(|ctx| {
            ctx.print(
                0 as UiValue,
                0 as UiValue,
                Span::styled(symbol.to_string(),
                             Style::default().fg(color_text),
                ),
            )
        });
    
    frame.render_widget(canvas, area);
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

// Layout::Rec symetric split via percent
//
fn split_area(input: Rect,
              direction: Direction,
              size: usize,
) -> Vec<Rect> {
    Layout::default()
        .direction(direction)
        .constraints(
            (0..size)
                .into_iter()
                .map(|_| Constraint::Percentage((100_usize / size) as u16))
                .collect::<Vec<_>>()
                .as_ref()
        )
        .split(input)
}

// Layout::Rec symetric split via percent
//
#[allow(unused)]
fn split_area_fixed(input: Rect,
                    direction: Direction,
                    len: usize,
                    size: u16,
) -> Vec<Rect> {
    Layout::default()
        .direction(direction)
        .constraints(
            (0..len)
                .into_iter()
                .map(|_| Constraint::Length(size))
                .collect::<Vec<_>>()
                .as_ref()
        )
        .split(input)
}

fn draw_bar<B>(color_index: &ColorIndex,
               chunks: Vec<Rect>,
               frame: &mut Frame<B>,
)
where
    B: Backend
{
    color_index
        .iter()
        .rev()
        //.step_by(20) // 500 / 20 -> 24 + 1
        // IRON_BOW
        //.step_by(9) // 119 / 9 -> 13 + 1
        // LONG
        .step_by(17) // 433 / 17 -> 24 + 1
        .enumerate()
        .for_each(|(index, color)| {
            if let Some(ch) = chunks.get(index as usize) {
                // CANVAS_BAR
                show_canvas_bar(
                    if FLAG_SHOW_BAR_INDEX.eq(&true) { format!("{index}|{:02.02}", color.0) } else { format!("{:02.02}", color.0) },
                    Color::Cyan,
                    Some(color.1),
                    frame,
                    *ch,
                );      
            }
        });
}


// map render: value + color
// two chunks as via single iter
//
fn draw_map<B>(chunks_lines_left: Vec<Rect>,
               chunks_lines_right: Vec<Rect>,
               array: Vec<Pixel>,
               frame: &mut Frame<B>,
)
where
    B: Backend
{
    // measure duration + async
    (0..LEN)
        .into_iter()
        .for_each(|row| {
            let chunks_cell_left = split_area(chunks_lines_left[row], Horizontal, LEN);

            // PERCENT
            let chunks_cell_right = split_area(chunks_lines_right[row], Horizontal, LEN);

            /* // LENGTH
            let chunks_cell_right = split_area_fixed(chunks_lines_right[row],
                                                     Horizontal,
                                                     LEN,
                                                     3_u16,
            );
            */
            
            (0..LEN)
                .into_iter()
                .for_each(|cell| {
                    let index = (row * LEN) + cell;
                    let pixel = match array.as_slice().get(index) {
                        Some(r) => *r,
                        None => Pixel {
                            index: index as UiValue,
                            value: TEMPERATURE_ERROR_SLICE_VALUE as UiValue,
                            boundary_variant: None,
                            color: None,
                        },
                    };
                    
                    // LEFT <- TEMPERATURE
                    if let Some(ch) = chunks_cell_left.get(cell) {
                        // CANVAS_VALUES
                        show_canvas_values(
                            pixel,
                            Color::Gray, // text
                            pixel.color, // bg
                            if FLAG_SHOW_INDEX.eq(&true) { ShowIndex::True } else { ShowIndex::False },
                            if FLAG_SHOW_COLOR.eq(&true) { ShowColor::True } else { ShowColor::False },
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

//
fn mqtt_payload_parse(sender: mpsc::Sender<ChannelData>) {
    thread::spawn(move || {
        let mqtt_uniq_id = format!("{}_{}",
                                   MQTT_CLIENT_ID,
                                   Uuid::new_v4().simple(),
        );
        
        let mut mqttoptions = MqttOptions::new(
            mqtt_uniq_id,
            MQTT_HOST,
            MQTT_PORT,
        );
        
        mqttoptions.set_credentials(MQTT_USER, MQTT_PASS);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        
        let (mut mqtt_client, mut mqtt_connection) = Client::new(mqttoptions, 10);
        
        match mqtt_client.subscribe(MQTT_TOPIC, QoS::AtMostOnce) {
            Ok(_status) => {},
            Err(e) => {
                panic!("mqtt subscibe failed: {e:?}");
            },
        }
        
        for (_, notification) in mqtt_connection.iter().enumerate() {
            match notification {
                Ok(rumqttc::Event::Incoming(Packet::Publish(publish_data))) => {
                    if publish_data.topic.eq(&MQTT_TOPIC) && publish_data.payload.len().eq(&MQTT_PAYLOAD_SIZE) {
                        
                        let mut max_value = TEMPERATURE_MAX;
                        let mut max_index: usize = 0;
                        
                        let mut min_value = TEMPERATURE_MIN;
                        let mut min_index: usize = 0;
                        
                        // PAYLOAD
                        let chunks = publish_data.payload.chunks(CHUNK_SIZE);
                        // todo!() measure duration and async ...
                        let array: [Temperature; LEN * LEN] = chunks
                            .enumerate()
                            .map(|(index, chunk)| {
                                let chunk_result: Result<[u8; 4], _> = chunk.try_into();
                                match chunk_result {
                                    Ok(value) => {
                                        let value = Temperature::from_be_bytes(value);
                                        
                                        if value > max_value {
                                            max_value = value;
                                            max_index = index;
                                        }
                                        
                                        if value < min_value {
                                            min_value = value;
                                            min_index = index;
                                        }
                                        
                                        value
                                    },
                                    Err(_e) => {
                                        TEMPERATURE_ERROR_VALUE
                                    },
                                }
                            })
                            .collect::<Vec<Temperature>>()
                            .try_into()
                            .unwrap();

                        sender.send(ChannelData {min_value,
                                                 min_index,
                                                 max_value,
                                                 max_index,
                                                 array,
                        }).unwrap()
                    }
                },
                Ok(_other) => {},
                Err(e) => panic!("mqtt notification failed: {e:?}"),
            }
        }
    });
}

//
fn index_color_pallete(boundary_max: Temperature,
                       boundary_min: Temperature,
) -> ColorIndex
{
    let range = (boundary_max - boundary_min) / PALLETE.len() as Temperature;

    PALLETE
        .iter()
        .enumerate()
        .map(|(index, color)| {
            // this keeps the lowest value inaccessible
            // so color will be COLOR_NONE_MAP
            // todo!() try harder
            let temperature = ((boundary_min as Temperature + (index + 1) as Temperature * range) / TEMPERATURE_INDEX_STEP).ceil() * TEMPERATURE_INDEX_STEP;
            
            (temperature, *color)
        })
        .collect::<Vec<(Temperature, (u8, u8, u8))>>()
}
