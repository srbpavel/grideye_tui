use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use uuid::Uuid;

use rumqttc::Client;
use rumqttc::Connection;
use rumqttc::MqttOptions;
use rumqttc::Packet;
use rumqttc::QoS;

// todo(!) -> move to config
const MQTT_HOST: &str = "192.168.0.103";
const MQTT_USER: &str = "";
const MQTT_PASS: &str = "";
const MQTT_PORT: u16 = 1883;
const MQTT_TOPIC: &str = "/grid_eye/";
const MQTT_CLIENT_ID: &str = "grideye_tui";
const MQTT_RECONNECT_DELAY: Duration = Duration::from_millis(1000);

const TEMPERATURE_ERROR_VALUE: f32 = 86.0;
const TEMPERATURE_MAX: f32 = -55.0;
const TEMPERATURE_MIN: f32 = 125.0;

pub const LEN: usize = 8;
pub const POW: usize = LEN * LEN;
const CHUNK_SIZE: usize = 4;
const MQTT_PAYLOAD_SIZE: usize = POW * CHUNK_SIZE;

pub type Temperature = f32;
pub type Array =  [Temperature; POW];

//
// payload data + boundary values
//
pub struct Payload {
    pub min_value: Temperature,
    pub min_index: usize,
    pub max_value: Temperature,
    pub max_index: usize,
    pub array: Array,
}

//
// - listen for incomming payload
// - reconnects if broker went down and up
// - mark max and min temperature values
// - send data via channel into main loop for rendering
//
pub struct Mqtt {
    client: Option<Client>,
    connection: Option<Connection>,
}

impl Mqtt {
    //
    pub fn connect() -> Self {
        let mqtt_uniq_id = format!("{}_{}",
                                   MQTT_CLIENT_ID,
                                   Uuid::new_v4().simple(),
        );
        
        let mut options = MqttOptions::new(
            mqtt_uniq_id,
            MQTT_HOST,
            MQTT_PORT,
        );

        options.set_credentials(MQTT_USER, MQTT_PASS);
        options.set_keep_alive(Duration::from_secs(5));

        let (client, connection) = Client::new(options, 10);

        Self {
            client: Some(client),
            connection: Some(connection),
        }
    }
    
    //
    pub fn subscribe(mut self) -> Self {
        if let Some(ref mut client) = self.client {
            match client.subscribe(MQTT_TOPIC, QoS::AtMostOnce) {
                Ok(_status) => {},
                Err(e) => {
                    // todo!() -> wait to for SUB
                    panic!("mqtt subscibe failed: {e:?}");
                },
            }
        }

        self
    }

    //
    pub fn parse(self,
             data_sender: mpsc::Sender<Payload>,
    ) {
        if let Some(mut mqtt_connection) = self.connection {
            thread::spawn(move || {
                for (_, event) in mqtt_connection.iter().enumerate() {
                    // type Item = Result<Event, ConnectionError>
                    match event {
                        //
                        // we verify topic + payload len
                        //
                        // Publish {
                        //    topic: String,
                        //    payload: Bytes,
                        //    ..
                        //}
                        Ok(rumqttc::Event::Incoming(Packet::Publish(publish_data))) => {
                            if publish_data.topic.eq(&MQTT_TOPIC) && publish_data.payload.len().eq(&MQTT_PAYLOAD_SIZE) {

                                // Payload boundary values
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
                                
                                data_sender.send(Payload {min_value,
                                                          min_index,
                                                          max_value,
                                                          max_index,
                                                          array,
                                }).unwrap()
                            }
                        },
                    Ok(_invalid_payload) => {
                        //todo(!) --> display invalid payload in log list
                    },
                        Err(_e) => { // todo(!) use ERROR + show error in log_list
                            thread::sleep(MQTT_RECONNECT_DELAY);
                            
                            thread::spawn(move || {
                                let mqtt = Mqtt::connect();
                                
                                mqtt
                                    .subscribe()
                                    .parse(data_sender);
                            });
                        
                            break // verify if this is not too much ???
                        },
                    }
                }
            });
        }
    }
}
