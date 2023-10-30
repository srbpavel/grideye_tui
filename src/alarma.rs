use crate::mqtt::Temperature;
use crate::config::DateTime;

#[derive(Clone)]
pub struct Alarma {
    pub max: Temperature,
    pub min: Temperature,
    pub diff: Temperature,
    pub datetime: DateTime,
}
