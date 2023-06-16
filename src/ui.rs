use crate::PALLETE;

use crate::pallete;
use crate::pallete::ColorIndex;
use crate::pallete::ColorRGB;

use crate::mqtt;
use crate::mqtt::LEN;
use crate::mqtt::POW;
use crate::mqtt::Temperature;
use crate::mqtt::Array;

use crate::app::App;

use std::collections::VecDeque;
use std::time::Duration;

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
use tui::style::Modifier;
use tui::symbols::Marker;
use tui::text::Span;
use tui::text::Spans;
use tui::widgets::Axis;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Chart;
use tui::widgets::Dataset;
use tui::widgets::canvas::Canvas;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::Cell;
use tui::widgets::Row;
use tui::widgets::Tabs;
use tui::widgets::Table;


// 1000ms / 25ms = 40fps/Hz
pub const UI_REFRESH_DELAY: Duration = Duration::from_millis(25);
pub const DATA_CAPACITY: usize = 100;
const TEMPERATURE_ERROR_SLICE_VALUE: f32 = 99.0;
const TEMPERATURE_DEFAULT_VALUE: f32 = 126.0;
const TEMPERATURE_MAX: f32 = -55.0;
const TEMPERATURE_MIN: f32 = 125.0;
const TEMPERATURE_BOUNDARY_OFFSET: f32 = 5.0;
pub const TEMPERATURE_INDEX_STEP: f32 = 0.25; // todo!() verify sensor resolution
const COLOR_NONE_MAP: Color = Color::Black;
const COLOR_NONE_BAR: Color = Color::Magenta;
const FLAG_SHOW_INDEX: bool = false;
const FLAG_SHOW_BAR_INDEX: bool = false;
const FLAG_SHOW_COLOR: bool = false;
const BAR_LEN: usize = PALLETE.len() / 17;

type UiValue = f64;

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

//
// data for rendering
//
//pub struct Data<'msg> {
pub struct Data {
    capacity: usize,
    array: Array,
    boundary_max: Boundary,
    boundary_min: Boundary,
    diff: Temperature,
    //log_messages: VecDeque<&'msg str>,
}

//impl<'msg> Default for Data<'msg> {
impl Default for Data {
    fn default() -> Self {
        Self {
            capacity: DATA_CAPACITY,
            array: [TEMPERATURE_DEFAULT_VALUE; POW],
            boundary_max: Boundary::new(BoundaryVariant::Max),
            boundary_min: Boundary::new(BoundaryVariant::Min),
            diff: 0 as Temperature,
            //log_messages: VecDeque::default(),
        }
    }
}

//impl<'msg> Data<'msg> {
impl Data {
    pub fn truncate(&mut self) {
        self.boundary_min.history.truncate(self.capacity);
        self.boundary_max.history.truncate(self.capacity);
    }

    //
    pub fn fill(&mut self,
                channel_data: mqtt::Payload,
    ) {

        self.array = channel_data.array;
        
        // DEFAULT dynamic
        self.boundary_min.history.push_front(channel_data.min_value);
        self.boundary_min.index = channel_data.min_index;
        self.boundary_min.value = channel_data.min_value;
        
        self.boundary_max.history.push_front(channel_data.max_value);
        self.boundary_max.index = channel_data.max_index;
        self.boundary_max.value = channel_data.max_value;
        
        self.diff = channel_data.max_value - channel_data.min_value;
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
                    color: pallete::temperature_to_color(
                        color_index.as_slice(),
                        *value as Temperature,
                    ),
                }
            })
            .collect::<Vec<_>>()
    }
}

// our inital frame
//
pub fn draw<B>(frame: &mut Frame<B>,
               app: &mut App,
               data: &mut Data,
)
where
    B: Backend
{
    // main window --> top tabs + bottom rest
    let chunks = Layout::default()
        .direction(Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0)
        ].as_ref())
        .split(frame.size());

    let tab_titles = app
        .tabs
        .titles
        .iter()
        .map(|t| {
            Spans::from(Span::styled(*t, Style::default().fg(Color::Green)))
        })
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default()
               .title(app.title)
               // todo(!) -> study + fix
               // !!! this hides TAB names
               //.borders(Borders::ALL)
        )
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(app.tabs.index);
    
    // draw tabs
    frame.render_widget(tabs, chunks[0]);
    
    // draw rest under
    match app.tabs.index {
        0 => draw_first_tab(frame, app, chunks[1], data),
        1 => draw_second_tab(frame, app, chunks[1]),
        2 => draw_third_tab(frame, app, chunks[1]),
         _ => {}
    };
}

//
fn draw_first_tab<B>(frame: &mut Frame<B>,
                     _app: &mut App,
                     area: Rect,
                     data: &Data)
where
    B: Backend,
{
    let color_index = pallete::index_color_pallete(
        data.boundary_max.value,
        data.boundary_min.value,
    );

    let pixel_array = data.pixels(&color_index);
    
    // --> top (graph + bar) + bottom (grid + map + log)
    let chunks = Layout::default()
        .direction(Vertical)
        .constraints([
            // /*
            Constraint::Percentage(80),
            Constraint::Percentage(20),
            // */
            /*
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2),
            */
            /*
            Constraint::Length(2),
            Constraint::Min(0)
            */
        ])
        .split(area);

    // top --> left graph + right bar
    let top = Layout::default()
        .direction(Horizontal)
        .constraints([
            Constraint::Percentage(97),
            Constraint::Percentage(3),
            /*
            Constraint::Ratio(1, 2),
            Constraint::Ratio(1, 2),
            */
            /*
            Constraint::Length(2),
            Constraint::Min(0)
            */
        ])
        .split(chunks[0]);

    // top --> right --> heat bar with many many canvas
    let chunks_top_right_bar = split_area(top[1],
                                          Vertical,
                                          BAR_LEN,
    );
    
    // bottom --> left (value) + right (color + log)
    let chunks_bottom = split_area(chunks[1], Horizontal, 2);

    // bottom --> right --> color map + log
    let chunks_right = Layout::default()
        .direction(Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ].as_ref())
        .split(chunks_bottom[1]);

    // bottom --> left --> value --> LEN * LEN canvas
    let chunks_lines_left = split_area(chunks_bottom[0], Vertical, LEN);
    // bottom --> right --> color_map --> LEN * LEN canvas
    let chunks_lines_right = split_area(chunks_right[0], Vertical, LEN);
    
    // max+min graph
    draw_chart(
        "", //"chart graph",
        data,
        Color::Red,
        Color::Cyan,
        frame,
        top[0]
    );

    // heat bar
    draw_bar(&color_index,
             chunks_top_right_bar,
             frame,
    );
    
    // grid + heatmap via single iter
    draw_map(chunks_lines_left,
             chunks_lines_right,
             pixel_array,
             frame,
    );

    // log info
    //draw_status(&data.log_messages,
    draw_status(["a1", "b2", "c3", "d4", "e5"].as_slice(),
                Color::DarkGray,
                Color::Black,
                frame,
                chunks_right[1],
    );
}

//
fn draw_second_tab<B>(_frame: &mut Frame<B>,
                      _app: &mut App,
                      _area: Rect)
where
    B: Backend,
{}

// learning sample
//
fn draw_third_tab<B>(frame: &mut Frame<B>,
                     _app: &mut App,
                     area: Rect)
where
    B: Backend,
{

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
fn draw_chart<B>(
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
        .block(Block::default()
               .title(title)
               .borders(Borders::ALL)
        )
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

    frame.render_widget(chart, area);
}

// draw heat bar
//
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
        .step_by(BAR_LEN)
        .enumerate()
        .for_each(|(index, color)| {
            if let Some(ch) = chunks.get(index as usize) {
                // CANVAS_BAR
                draw_canvas_bar(
                    if FLAG_SHOW_BAR_INDEX.eq(&true) { format!("{index}|{:02.02}", color.0) } else { format!("{:02.02}", color.0) },
                    Color::Cyan,
                    Some(color.1),
                    frame,
                    *ch,
                );      
            }
        });
}

// draw color for each temperature
//
fn draw_canvas_bar<B>(
    symbol: String,
    color_text: Color,
    color_area: Option<ColorRGB>,
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
    // todo(!) --> measure duration + async
    (0..LEN)
        .into_iter()
        .for_each(|row| {
            // todo(!) --> this two can go async
            let chunks_cell_left = split_area(chunks_lines_left[row],
                                              Horizontal,
                                              LEN);

            let chunks_cell_right = split_area(chunks_lines_right[row],
                                               Horizontal,
                                               LEN);

            // todo(!) --> try rayon for first time ???
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

// display logs
//
fn draw_status<B>(
    //msg: &VecDeque<&str>,
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

// display single pixel as value
// min/max values are highlighted with color
//
fn show_canvas_values<B>(
    pixel: Pixel,
    color_text: Color,
    color_area: Option<ColorRGB>,
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
    color_area: Option<ColorRGB>,
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
fn split_area(input: Rect,
              direction: Direction,
              size: usize,
) -> Vec<Rect> {
    Layout::default()
        .direction(direction)
        .constraints(
            (0..size)
                .into_iter()
                .map(|_| Constraint::Ratio(1, size as u32))
                .collect::<Vec<_>>()
                .as_ref()
        )
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
