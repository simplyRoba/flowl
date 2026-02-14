use rumqttc::{AsyncClient, Event, MqttOptions, Packet};
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::config::Config;

pub struct MqttHandle {
    pub client: AsyncClient,
    task: JoinHandle<()>,
}

impl MqttHandle {
    pub async fn disconnect(self) {
        if let Err(e) = self.client.disconnect().await {
            warn!("MQTT disconnect error: {e}");
        }
        self.task.abort();
    }
}

pub fn connect(config: &Config) -> MqttHandle {
    let client_id = format!("{}-{}", config.mqtt_topic_prefix, std::process::id());
    let mut options = MqttOptions::new(&client_id, &config.mqtt_host, config.mqtt_port);
    options.set_keep_alive(std::time::Duration::from_secs(30));

    let (client, mut event_loop) = AsyncClient::new(options, 10);

    let task = tokio::spawn(async move {
        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    info!("MQTT connected");
                }
                Ok(_) => {}
                Err(e) => {
                    warn!("MQTT connection error: {e}");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    });

    MqttHandle { client, task }
}
