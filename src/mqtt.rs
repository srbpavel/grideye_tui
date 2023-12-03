#[allow(unused)]
use crate::info;
#[allow(unused)]
use crate::warn;
#[allow(unused)]
use crate::error;

use crate::config;
use crate::config::Config;

use crate::ui;
use crate::ui::BoundaryVariant;
use crate::run;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::collections::VecDeque;

use bytes::Bytes;

use rumqttc::Client;
use rumqttc::Connection;
use rumqttc::Event;
use rumqttc::mqttbytes::v4::Packet;
use rumqttc::QoS;

pub const LEN: usize = 8;
pub const POW: usize = LEN * LEN;
const CHUNK_SIZE: usize = 4;
const MQTT_PAYLOAD_SIZE: usize = POW * CHUNK_SIZE;

const MQTT_RECONNECT_DELAY: Duration = Duration::from_millis(1000);
const TEMPERATURE_ERROR_VALUE: f32 = 86.0;
const TEMPERATURE_MAX: f32 = -55.0;
const TEMPERATURE_MIN: f32 = 125.0;

// todo(!) cfg
//const ERROR_CAPACITY: usize = 100;
const COMMON_LOG_CAPACITY: usize = 40;

pub type Temperature = f32;
pub type Array =  [Temperature; POW];

#[derive(Clone)]
pub struct CommonMsg {
    pub datetime: config::DateTime,
    pub entry: String,
}

impl Default for CommonMsg {
    fn default() -> Self {
        Self {
            datetime: config::now(),
            entry: String::from("default_entry"),
        }
    }
}

impl CommonMsg {
    // via mqtt Payload
    pub fn new(data: Bytes) -> Self {
        Self {
            entry: data
                .iter()
                .map(|c| String::from(*c as char))
                .collect::<String>(),
            ..Default::default()
        }
    }

    // via text
    pub fn record(entry: String) -> Self {
        Self {
            entry, 
            ..Default::default()
        }
    }
}

impl std::fmt::Display for CommonMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} --> {}",
               self.datetime,
               self.entry,
        )
    }
}

pub struct CommonLog {
    pub logs: VecDeque<CommonMsg>,
}

impl Default for CommonLog {
    fn default() -> Self {
        Self {
            logs: VecDeque::with_capacity(COMMON_LOG_CAPACITY),
        }
    }
}

impl CommonLog {
    //
    pub fn truncate(&mut self) {
        self.logs.truncate(COMMON_LOG_CAPACITY);
    }

    //
    pub fn add(&mut self,
               msg: CommonMsg,
    ) {
        self.logs.push_front(msg.clone());
        self.log(msg);
    }

    //
    fn log(&mut self,
            msg: CommonMsg,
    ) {
        info!("{}", msg);
    }
}

pub struct PayloadBoundary {
    pub index: usize,
    pub value: Temperature,
    pub variant: BoundaryVariant,
}

impl PayloadBoundary {
    //
    fn update(&mut self,
              value: Temperature,
              index: usize,
    ) {
        match self.variant {
            BoundaryVariant::Min => {
                if value < self.value {
                    self.value = value;
                    self.index = index;
                }       
            },
            BoundaryVariant::Max => {
                if value > self.value {
                    self.value = value;
                    self.index = index;
                }
            },
        }
    }

    //
    fn init(variant: BoundaryVariant) -> Self {
        match variant {
            BoundaryVariant::Min => {
                Self {
                    index: 0,
                    value: TEMPERATURE_MIN,
                    variant,
                }
            },
            BoundaryVariant::Max => {
                Self {
                    index: 0,
                    value: TEMPERATURE_MAX,
                    variant,
                }
            },
        }
    }
}

//
// payload + boundary values
//
pub struct Payload {
    pub topic: String,
    pub min: PayloadBoundary,
    pub max: PayloadBoundary,
    pub array: Array,
    pub datetime: config::DateTime,
    pub raw: Bytes,
    
}

impl Payload {
    //
    pub fn verify_topic(&self,
                        config: &Config,
    ) -> Option<(ui::Topic, ui::Uuid)> {
        let topic_parts = self.topic.split(config.topic_delimiter);
        
        let count = topic_parts
            .clone()
            .count();

        if count.eq(&config.topic_parts_count) {
            let mut topic: Vec<&str> = topic_parts.collect();
            
            let uuid = topic
                .pop() // this pop's uuid
                .unwrap() //should be always good as len is verified?
                .to_string();
            
            let topic_parent = topic.join(config.topic_delimiter);

            /* // too many logs
            info!("verify_topic(): {}",
                  topic_parent,
            );
            */
            
            Some((topic_parent, uuid))
        } else {
            info!("verify_topic(): count is not eq[{}]: {:?}",
                  count,
                  topic_parts,
            );
            
            None
        }
    }

    //
    pub fn parse(self,
                 config: &Config,
                 devices: &mut run::Devices,
    ) {
        match self.verify_topic(config) {
            Some((mut topic_parent, uuid)) => {
                // ???
                if topic_parent.contains("simulator") {
                    topic_parent = self
                        .topic
                        .clone();
                }
                
                match devices.get_mut(&topic_parent) {
                    // update
                    Some(single_device) => {
                        /* // to many logs
                        info!("payload::parse(): device update -> {}",
                              topic_parent,
                        );
                        */

                        //negative min value
                        if self.min.value.le(&0.0) {
                            info!("NEGATIVE >>> topic: {} value: {:?}\npayload: {:?}",
                                  topic_parent,
                                  self.min.value,
                                  self.array,
                            )
                        }
                        //
                        
                        single_device.fill(config,
                                           self,
                                           topic_parent,
                                           uuid,
                        );
                        
                        single_device.truncate();
                    },
                    // init
                    None => {
                        info!("payload::parse(): device init -> {}",
                              topic_parent,
                        );

                        if let Some(mut single_device) =
                            devices.insert(topic_parent.clone(),
                                           ui::Device::init(config),
                            ) {
                            single_device.fill(config,
                                               self,
                                               topic_parent,
                                               uuid,
                            );
                        }
                    },
                }
            },
            None => {
                error!("wrong topic format/len: {:?}",
                       self.topic,
                );
            }
        }
    }
}

//
// - listen for incomming payload
// - reconnects if broker went down and up
// - mark max and min temperature values
// - send data via channel into main loop for rendering
//
pub struct Mqtt {
    config: Config,
    client: Option<Client>,
    connection: Option<Connection>,
}

impl Mqtt {
    //
    pub fn new(config: Config) -> Self {
        Self {
            config,
            client: None,
            connection: None,
        }
    }

    //
    pub fn connect(mut self) -> Self {
        let mut options = rumqttc::MqttOptions::new(
            uniq_id(self.config.mqtt_client_id),
            self.config.mqtt_broker_url,
            self.config.mqtt_port,
        );
        
        options.set_credentials(self.config.mqtt_user, self.config.mqtt_pass);
        options.set_keep_alive(Duration::from_secs(5));
        info!("MQTT Options: {options:?}");
        
        let (client, connection) = Client::new(options.clone(), 10);

        self.client = Some(client);
        self.connection = Some(connection);

        self
    }
    
    //
    pub fn subscribe(mut self) -> Self {
        let mqtt_topic = create_topic(self.config.mqtt_topic_base,
                                      &[self.config.mqtt_topic_wildcard],
        );
        
        info!("MQTT topic to sub: {mqtt_topic:?}");
        
        if let Some(ref mut client) = self.client {
            match client.subscribe(mqtt_topic, QoS::AtMostOnce) {
                Ok(_) => {
                    info!("MQTT Sub");
                },
                Err(e) => {
                    // todo!() -> wait to for SUB
                    //panic!("mqtt subscibe failed: {e:?}");
                    error!("MQTT Sub Error : {e}");
                },
            }
        }

        self
    }

    //
    pub fn parse(self,
                 data_sender: mpsc::Sender<Payload>,
                 common_sender: mpsc::Sender<CommonMsg>,
    ) {
        if let Some(mut mqtt_connection) = self.connection {
            let mqtt_topic_error =
                create_topic(self.config.mqtt_topic_base,
                             &[self.config.mqtt_topic_error_suffix],
                );

            thread::spawn(move || {
                for event in mqtt_connection.iter() {
                    // // type Item = Result<Event, ConnectionError>
                    match event {
                        /*
                        Publish {
                            dup: bool,
                            qos: QoS,
                            retain: bool,
                            topic: String,
                            pkid: u16,
                            payload: Bytes,
                        */
                        Ok(Event::Incoming(Packet::Publish(publish_data))) => {
                            // DEBUG
                            /*
                            info!("MQTT incomming data: {:?}",
                                  publish_data.topic,
                            );
                            */
                            
                            // verify topic + payload len
                            if publish_data.topic.contains(self.config.mqtt_topic_base) && publish_data.payload.len().eq(&MQTT_PAYLOAD_SIZE) {

                                let payload = parse_incomming(publish_data);
                            
                                data_sender
                                    .send(payload)
                                    .unwrap()
                            } else if publish_data.topic.eq(&mqtt_topic_error) {
                                common_sender
                                    .send(
                                        CommonMsg::new(publish_data.payload)
                                    )
                                    .unwrap();
                            } else {
                                error!("invalid payload: {:#?}",
                                       publish_data,
                                );
                            }
                        },
                        Ok(invalid_payload) => {
                            match invalid_payload {
                                Event::Incoming(Packet::PingResp) => {},
                                Event::Outgoing(rumqttc::Outgoing::PingReq) => {},
                                _ => {
                                    error!("invalid payload: {:?}",
                                           invalid_payload,
                                    );
                                },
                            }
                        },
                        // todo(!) -> verify again
                        Err(e) => { // todo(!) use ERROR + show error in log_list
                            error!("MQTT Event Error: {e}");

                            thread::sleep(MQTT_RECONNECT_DELAY);
                            
                            thread::spawn(move || {
                                Mqtt::new(self.config)
                                    .connect()
                                    .subscribe()
                                    .parse(data_sender,
                                           common_sender,
                                    );
                            });
                            
                            break // verify if this is not too much ???
                        },
                    }
                }
            });
        }
    }
}

//
fn parse_incomming(publish_data: rumqttc::Publish,
) -> Payload {
    // DEBUG
    //info!("topic: {}", publish_data.topic);
    
    let now = config::now();
    
    // init boundary values
    let mut max = PayloadBoundary::init(BoundaryVariant::Max);
    let mut min = PayloadBoundary::init(BoundaryVariant::Min);
    
    // Payload
    let raw = publish_data.payload.clone();
    let chunks = publish_data.payload.chunks(CHUNK_SIZE);
    let array: [Temperature; LEN * LEN] = chunks.clone()
        .enumerate()
        .map(|(index, chunk)| {
            let chunk_result: Result<[u8; 4], _> = chunk.try_into();

            match chunk_result {
                Ok(value) => {
                    let value = Temperature::from_be_bytes(value);
                    
                    min.update(value, index);
                    max.update(value, index);

                    // DEBUG RAW
                    if value.le(&0.0) {
                        info!("NEGATIVE raw >>> index: {}, chunk: {:?}",
                              index,
                              chunk,
                        )
                    }
                    
                    value
                },
                Err(e) => {
                    error!("not valid chunk_result: {e:?}\nchunk: {:?}\nchunks: {:?}",
                           chunks,
                           chunk_result,
                    );

                    TEMPERATURE_ERROR_VALUE
                },
            }
        })
        .collect::<Vec<Temperature>>()
        .try_into()
        .unwrap();  // niet goed !!!
    
    /* // MEASURE
    // can also use Instant
    warn!("measure payload -> array: {:?}",
          /*
          0
          (Utc::now() - now).num_millisecond(),
          
          Some(1)..Some(6)
          Utc::now() - now).num_microseconds(),
          
          Some(1056)..Some(2253)
          */
          //(Utc::now() - now).num_nanoseconds(),
          (config::now() - now).num_nanoseconds(),
    );
    */
    
    Payload {
        topic: publish_data.topic,
        min,
        max,
        array,
        datetime: now,
        // DEBUG
        raw,
    }
}

//
fn uniq_id(prefix: &str) -> String {
    format!("{}_{}",
            prefix,
            config::uuid(),
    )
}

// todo! try harder
//
pub fn create_topic(base: &str,
                    parts: &[&str],
) -> String {

    let mut path = std::path::PathBuf::new();
    path.push(base);
    
    let topic = parts
        .iter()
        .fold(path, |topic, part|
              topic.join(part)
        );

    match topic
        .to_str() {
            Some(t) => String::from(t),
            None => {
                //todo! dispaly error
                //todo! decide what now 
                format!("error create_topic to_str() -> {}",
                        base,
                )
            },
        }
}

