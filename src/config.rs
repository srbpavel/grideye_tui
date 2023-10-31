// if cfg.toml is wrong it will quietly use default values !!!
#[allow(unused)]
use crate::info;
#[allow(unused)]
use crate::warn;
#[allow(unused)]
use crate::error;

use chrono::Duration;

// value for Duration
type Limit = i64;

#[derive(Clone)]
#[toml_cfg::toml_config]
pub struct Config {
    #[default("default_app_title")]
    app_title: &'static str,

    #[default("Utc")]
    datetime_timezone: &'static str,

    #[default("localhost")]
    mqtt_broker_url: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_pass: &'static str,
    #[default(1883)]
    mqtt_port: u16,
    #[default("/default_mqtt_topic_base")]
    mqtt_topic_base: &'static str,
    #[default("default_mqtt_topic_error_suffix")]
    mqtt_topic_error_suffix: &'static str,
    #[default("#")]
    mqtt_topic_wildcard: &'static str,
    #[default("default_client_id_")]
    mqtt_client_id: &'static str,

    #[default("/")]
    topic_delimiter: &'static str,
    #[default(4)]
    topic_parts_count: usize,

    #[default(false)]
    flag_show_bar_index: bool,
    #[default(false)]
    flag_show_index: bool,
    #[default(false)]
    flag_show_color: bool,
    #[default(false)]
    flag_show_map_table_index: bool,

    #[default(5.0)]
    alarma_diff: f32,
        
    #[default("default_empty_topic")]
    default_empty_topic: &'static str,
    #[default("default_empty_uuid")]
    default_empty_uuid: &'static str,

    #[default(1000)]
    duration_limit_device_remove: Limit,
    #[default(500)]
    duration_limit_device_remove_warn: Limit,
    #[default(100)]
    duration_limit_device_unknown: Limit,

    #[default("default_grideye_ratatui_error_msg.log")]
    log_file: &'static str,
}

/*
#[derive(Debug)]
pub enum TimeZone {
    Utc(chrono::DateTime<chrono::Utc>),
    Local(chrono::DateTime<chrono::Local>),
}
*/

impl Config {
    /*
    pub fn kolik<T>(&self,
                    x: T) -> chrono::DateTime<T>
    where T: Select + chrono::TimeZone,
    {
        chrono::Local::now()
        //chrono::T::now()
        //x.choose()
    }
    */
    
    /* //
    pub fn ted_hned(&self) -> TimeZone {
        match self.datetime_timezone {
            "utc" => TimeZone::Utc(_tedka(chrono::Utc::now())),
            "local" => TimeZone::Local(_tedka(chrono::Local::now())),
            _ => TimeZone::Utc(_tedka(chrono::Utc::now())),
        }
    }
    */
    
    //
    pub fn duration_limit_device_remove(&self) -> Duration {
        number_to_duration(self.duration_limit_device_remove)
    }

    //
    pub fn duration_limit_device_remove_warn(&self) -> Duration {
        number_to_duration(self.duration_limit_device_remove_warn)
    }

    //
    pub fn duration_limit_device_unknown(&self) -> Duration {
        number_to_duration(self.duration_limit_device_unknown)
    }
}

//just to have on single place in case of swap to Local
//
// try harder to have in conf !!!
//
//pub type DateTime = chrono::DateTime<chrono::Utc>;
pub type DateTime = chrono::DateTime<chrono::Local>;

/*
fn _select<T>(x: T) -> T
where
    T: Select,
{
    x.select()
}
*/

//
pub fn now() -> DateTime {
    //chrono::Utc::now()
    chrono::Local::now()
}

//
pub fn uuid() -> uuid::fmt::Simple {
    uuid::Uuid::new_v4().simple()
}

//
fn number_to_duration(value: i64) -> Duration {
    chrono::Duration::milliseconds(value)
}


trait Choose {
    fn choose(&self) -> Self
    where
        Self: Sized;
}

impl Choose for chrono::DateTime<chrono::Utc> {
    fn choose(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

impl Choose for chrono::DateTime<chrono::Local> {
    fn choose(&self) -> chrono::DateTime<chrono::Local> {
        chrono::Local::now()
    }
}


//
// x je treba nejaky typ Utc nebo Local
//
fn _tedka<T>(x: T) -> T
where
    T: Choose,
{
    x.choose()
}

trait Select {
    fn select(&self) -> Self
    where
        Self: Sized;
}

impl Select for chrono::Utc {
    fn select(&self) -> chrono::Utc {
        chrono::Utc
    }
}

impl Select for chrono::Local {
    fn select(&self) -> chrono::Local {
        chrono::Local
    }
}

fn _select<T>(x: T) -> T
where
    T: Select,
{
    x.select()
}

