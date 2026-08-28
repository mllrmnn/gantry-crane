use std::sync::Arc;

use crate::{
    constants::{AVAILABILITY_TOPIC, SET_STATE_TOPIC},
    events::{ContainerEventInfo, Event, EventSender},
    mqtt::MqttMessage,
    settings::Settings,
};

use super::{
    button::Button,
    device::Device,
    entity::{normalize_object_id, normalize_unique_id_part, DeviceClass, Entity, StateClass},
    sensor::Sensor,
};

pub struct HomeAssistantContainer {
    settings: Arc<Settings>,
    event_tx: EventSender,
    node_id: String,
    container_name: String,
    object_id_prefix: String,
    unique_id_prefix: String,
    is_host_networking: bool,
    device: Arc<Device>,
    sensors: Vec<Sensor>,
    buttons: Vec<Button>,
}

impl HomeAssistantContainer {
    pub fn new(
        settings: Arc<Settings>,
        event_tx: EventSender,
        container: &ContainerEventInfo,
        node_id: String,
    ) -> Self {
        let container_name = container.name.trim_start_matches('/').to_owned();
        let unique_id_prefix = format!(
            "{}_{}",
            normalize_unique_id_part(&node_id),
            normalize_unique_id_part(&container_name)
        );
        let object_id_prefix = normalize_object_id(&format!("{}_{}", node_id, container_name));
        let device_name = format!("{} ({})", container_name, node_id);

        Self {
            settings,
            event_tx,
            node_id,
            container_name,
            object_id_prefix,
            unique_id_prefix: unique_id_prefix.clone(),
            is_host_networking: container.is_host_networking,
            device: Arc::new(Device::new_with_identifier(
                device_name,
                format!("gc_{}", unique_id_prefix),
                Some("Docker".into()),
            )),
            sensors: Vec::new(),
            buttons: Vec::new(),
        }
        .setup_sensors()
        .setup_buttons()
    }

    fn state_topic(&self) -> String {
        format!("{}/{}", self.settings.mqtt.base_topic, self.container_name)
    }

    fn set_topic(&self) -> String {
        format!(
            "{}/{}/{}",
            self.settings.mqtt.base_topic, self.container_name, SET_STATE_TOPIC
        )
    }

    fn availability_topic(&self) -> Option<String> {
        Some(format!(
            "{}/{}",
            self.settings.mqtt.base_topic, AVAILABILITY_TOPIC
        ))
    }

    fn object_id(&self, suffix: &str) -> String {
        format!("{}_{}", self.object_id_prefix, suffix)
    }

    fn unique_id(&self, suffix: &str) -> String {
        format!("gc_{}_{}", self.unique_id_prefix, suffix)
    }

    fn sensor(
        &self,
        name: &str,
        suffix: &str,
        icon: Option<&str>,
        state_class: Option<StateClass>,
        unit_of_measurement: Option<&str>,
        value_template: &str,
        enabled_by_default: bool,
    ) -> Sensor {
        Sensor {
            availability_topic: self.availability_topic(),
            device: self.device.clone(),
            enabled_by_default: Some(enabled_by_default),
            entity_category: None,
            expire_after: None,
            force_update: None,
            icon: icon.map(Into::into),
            name: name.into(),
            default_entity_id: Some(self.object_id(suffix)),
            object_id: Some(self.object_id(suffix)),
            payload_available: None,
            payload_not_available: None,
            state_class,
            state_topic: self.state_topic(),
            unique_id: Some(self.unique_id(suffix)),
            unit_of_measurement: unit_of_measurement.map(Into::into),
            value_template: Some(value_template.into()),
        }
    }

    fn button(
        &self,
        name: &str,
        suffix: &str,
        payload_press: &str,
        icon: Option<&str>,
        device_class: Option<DeviceClass>,
        enabled_by_default: bool,
    ) -> Button {
        Button {
            availability_topic: self.availability_topic(),
            command_topic: self.set_topic(),
            device: self.device.clone(),
            device_class,
            enabled_by_default: Some(enabled_by_default),
            entity_category: None,
            icon: icon.map(Into::into),
            name: name.into(),
            default_entity_id: Some(self.object_id(suffix)),
            object_id: Some(self.object_id(suffix)),
            payload_available: None,
            payload_not_available: None,
            payload_press: payload_press.into(),
            retain: None,
            unique_id: Some(self.unique_id(suffix)),
        }
    }

    fn setup_sensors(mut self) -> Self {
        // Image
        self.sensors.push(self.sensor(
            "Image",
            "image",
            Some("mdi:docker"),
            None,
            None,
            "{{ value_json['image'] }}",
            true,
        ));

        // State
        self.sensors.push(self.sensor(
            "State",
            "state",
            Some("mdi:docker"),
            None,
            None,
            "{{ value_json['state'] }}",
            true,
        ));

        // Health
        self.sensors.push(self.sensor(
            "Health",
            "health",
            Some("mdi:heart-pulse"),
            None,
            None,
            "{{ value_json['health'] }}",
            true,
        ));

        // CPU
        self.sensors.push(self.sensor(
            "CPU Percentage",
            "cpu",
            Some("mdi:cpu-64-bit"),
            Some(StateClass::Measurement),
            Some("%"),
            "{{ value_json['cpu_percentage'] }}",
            true,
        ));

        // 1CPU
        self.sensors.push(self.sensor(
            "1CPU",
            "cpu_1core",
            Some("mdi:cpu-64-bit"),
            Some(StateClass::Measurement),
            Some("%"),
            "{{ value_json['cpu_1core_percentage'] }}",
            true,
        ));

        // Memory percentage
        self.sensors.push(self.sensor(
            "Memory Percentage",
            "mem",
            Some("mdi:memory"),
            Some(StateClass::Measurement),
            Some("%"),
            "{{ value_json['mem_percentage'] }}",
            true,
        ));

        // Memory absolute
        self.sensors.push(self.sensor(
            "Memory Usage",
            "mem_mb",
            Some("mdi:memory"),
            Some(StateClass::Measurement),
            Some("MB"),
            "{{ value_json['mem_mb'] }}",
            true,
        ));

        // Network stats are not available in case of host network mode
        if !self.is_host_networking {
            // Net RX
            self.sensors.push(self.sensor(
                "Net RX",
                "net_rx",
                Some("mdi:download-network-outline"),
                Some(StateClass::Measurement),
                Some("MB"),
                "{{ value_json['net_rx_mb'] }}",
                false,
            ));

            // Net TX
            self.sensors.push(self.sensor(
                "Net TX",
                "net_tx",
                Some("mdi:upload-network-outline"),
                Some(StateClass::Measurement),
                Some("MB"),
                "{{ value_json['net_tx_mb'] }}",
                false,
            ));
        }

        // Block RX
        self.sensors.push(self.sensor(
            "Block RX",
            "block_rx",
            Some("mdi:file-download-outline"),
            Some(StateClass::Measurement),
            Some("MB"),
            "{{ value_json['block_rx_mb'] }}",
            false,
        ));

        // Block TX
        self.sensors.push(self.sensor(
            "Block TX",
            "block_tx",
            Some("mdi:file-upload-outline"),
            Some(StateClass::Measurement),
            Some("MB"),
            "{{ value_json['block_tx_mb'] }}",
            false,
        ));

        self
    }

    fn setup_buttons(mut self) -> Self {
        // Start
        self.buttons
            .push(self.button("Start", "start", "start", Some("mdi:play"), None, true));

        // Stop
        self.buttons
            .push(self.button("Stop", "stop", "stop", Some("mdi:stop"), None, true));

        // Restart
        self.buttons.push(self.button(
            "Restart",
            "restart",
            "restart",
            None,
            Some(DeviceClass::Restart),
            true,
        ));

        // Pause
        self.buttons
            .push(self.button("Pause", "pause", "pause", Some("mdi:pause"), None, false));

        // Unpause
        self.buttons.push(self.button(
            "Unpause",
            "unpause",
            "unpause",
            Some("mdi:play-pause"),
            None,
            false,
        ));

        // Recreate
        self.buttons.push(self.button(
            "Recreate",
            "recreate",
            "recreate",
            Some("mdi:autorenew"),
            None,
            false,
        ));

        // Pull and recreate
        self.buttons.push(self.button(
            "Pull and Recreate",
            "pull_recreate",
            "pull_recreate",
            Some("mdi:update"),
            None,
            false,
        ));

        self
    }

    pub fn publish(&self) {
        for sensor in &self.sensors {
            self.publish_entity(sensor);
        }
        for button in &self.buttons {
            self.publish_entity(button);
        }
    }

    fn publish_entity(&self, entity: &impl Entity) {
        match serde_json::to_string(&entity) {
            Ok(json) => {
                if let Err(e) = self
                    .event_tx
                    .send(Event::PublishMqttMessage(MqttMessage::new(
                        entity.topic(&self.settings.homeassistant.base_topic, &self.node_id),
                        json,
                        true,
                        1,
                    )))
                {
                    log::error!("Failed to publish MQTT message: {}", e);
                }
            }
            Err(e) => log::error!("Failed to serialize HA container: {}", e),
        }
    }

    pub fn unpublish(&self) {
        for sensor in &self.sensors {
            self.unpublish_entity(sensor);
        }
        for button in &self.buttons {
            self.unpublish_entity(button);
        }
    }

    fn unpublish_entity(&self, entity: &impl Entity) {
        if let Err(e) = self
            .event_tx
            .send(Event::PublishMqttMessage(MqttMessage::new(
                entity.topic(&self.settings.homeassistant.base_topic, &self.node_id),
                "".into(),
                true,
                1,
            )))
        {
            log::error!("Failed to publish MQTT message: {}", e);
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::{
        events::{ContainerEventInfo, Event, EventChannel},
        settings::Settings,
    };

    use super::HomeAssistantContainer;

    #[test]
    fn test_new() {
        let settings = temp_env::with_var_unset("DUMMY", || Settings::new(None).unwrap());
        let event_channel = EventChannel::new();
        let container = HomeAssistantContainer::new(
            Arc::new(settings),
            event_channel.get_sender(),
            &ContainerEventInfo {
                name: "/Container Name".into(),
                is_host_networking: false,
            },
            "node_id".into(),
        );

        assert_eq!(container.device.name, "Container Name (node_id)");
        assert_eq!(
            container.device.identifiers,
            vec!["gc_node_id_container_name".to_owned()]
        );
        assert_eq!(container.sensors.len(), 11);
        assert_eq!(container.buttons.len(), 7);
    }

    #[test]
    fn test_multi_host_ids_for_same_container_name() {
        let host_a = get_container("docker01-rrnuc", "/dockerproxy", false);
        let host_b = get_container("docker01-muestation", "/dockerproxy", false);

        assert_ne!(host_a.device.identifiers, host_b.device.identifiers);
        assert_eq!(
            host_a.device.identifiers,
            vec!["gc_docker01-rrnuc_dockerproxy".to_owned()]
        );
        assert_eq!(
            host_b.device.identifiers,
            vec!["gc_docker01-muestation_dockerproxy".to_owned()]
        );
        assert_eq!(host_a.device.name, "dockerproxy (docker01-rrnuc)");
        assert_eq!(host_b.device.name, "dockerproxy (docker01-muestation)");

        for (a, b) in host_a.sensors.iter().zip(host_b.sensors.iter()) {
            assert_ne!(a.unique_id, b.unique_id);
            assert_ne!(a.object_id, b.object_id);
            assert_eq!(a.state_topic, "gantry-crane/dockerproxy");
        }

        for (a, b) in host_a.buttons.iter().zip(host_b.buttons.iter()) {
            assert_ne!(a.unique_id, b.unique_id);
            assert_ne!(a.object_id, b.object_id);
            assert_eq!(a.command_topic, "gantry-crane/dockerproxy/set");
        }
    }

    #[test]
    fn test_multi_host_ids_for_different_containers_on_same_host() {
        let dockerproxy = get_container("docker01-rrnuc", "/dockerproxy", false);
        let portainer = get_container("docker01-rrnuc", "/portainer", false);

        assert_ne!(dockerproxy.device.identifiers, portainer.device.identifiers);
        assert_ne!(
            dockerproxy.sensors[0].unique_id,
            portainer.sensors[0].unique_id
        );
    }

    #[test]
    fn test_representative_entity_ids_and_defaults() {
        let container = get_container("docker01-rrnuc", "/dockerproxy", false);

        assert_sensor(
            &container,
            "Image",
            "gc_docker01-rrnuc_dockerproxy_image",
            "docker01_rrnuc_dockerproxy_image",
            true,
        );
        assert_sensor(
            &container,
            "Health",
            "gc_docker01-rrnuc_dockerproxy_health",
            "docker01_rrnuc_dockerproxy_health",
            true,
        );
        assert_sensor(
            &container,
            "CPU Percentage",
            "gc_docker01-rrnuc_dockerproxy_cpu",
            "docker01_rrnuc_dockerproxy_cpu",
            true,
        );
        assert_sensor(
            &container,
            "1CPU",
            "gc_docker01-rrnuc_dockerproxy_cpu_1core",
            "docker01_rrnuc_dockerproxy_cpu_1core",
            true,
        );
        assert_sensor(
            &container,
            "Memory Usage",
            "gc_docker01-rrnuc_dockerproxy_mem_mb",
            "docker01_rrnuc_dockerproxy_mem_mb",
            true,
        );
        assert_sensor(
            &container,
            "Net RX",
            "gc_docker01-rrnuc_dockerproxy_net_rx",
            "docker01_rrnuc_dockerproxy_net_rx",
            false,
        );
        assert_button(
            &container,
            "Restart",
            "gc_docker01-rrnuc_dockerproxy_restart",
            "docker01_rrnuc_dockerproxy_restart",
            true,
        );
    }

    #[test]
    fn test_publish() {
        let settings = temp_env::with_var_unset("DUMMY", || Settings::new(None).unwrap());
        let event_channel = EventChannel::new();
        let container = HomeAssistantContainer::new(
            Arc::new(settings),
            event_channel.get_sender(),
            &ContainerEventInfo {
                name: "/Container Name".into(),
                is_host_networking: false,
            },
            "node_id".into(),
        );

        let recv = event_channel.get_receiver();
        assert_eq!(recv.len(), 0);
        container.publish();
        assert_eq!(recv.len(), 18);
    }

    #[tokio::test]
    async fn test_publish_entity() {
        let settings = temp_env::with_var_unset("DUMMY", || Settings::new(None).unwrap());
        let event_channel = EventChannel::new();
        let container = HomeAssistantContainer::new(
            Arc::new(settings),
            event_channel.get_sender(),
            &ContainerEventInfo {
                name: "/Container Name".into(),
                is_host_networking: false,
            },
            "node_id".into(),
        );
        let entity = &container.sensors[0];

        let mut recv = event_channel.get_receiver();
        assert_eq!(recv.len(), 0);
        container.publish_entity(entity);
        assert_eq!(recv.len(), 1);

        let event = recv.recv().await.unwrap();
        assert!(matches!(event, Event::PublishMqttMessage { .. }));
        if let Event::PublishMqttMessage(msg) = event {
            assert_ne!(msg.payload.len(), 0);
        }
    }

    #[test]
    fn test_unpublish() {
        let settings = temp_env::with_var_unset("DUMMY", || Settings::new(None).unwrap());
        let event_channel = EventChannel::new();
        let container = HomeAssistantContainer::new(
            Arc::new(settings),
            event_channel.get_sender(),
            &ContainerEventInfo {
                name: "/Container Name".into(),
                is_host_networking: false,
            },
            "node_id".into(),
        );

        let recv = event_channel.get_receiver();
        assert_eq!(recv.len(), 0);
        container.unpublish();
        assert_eq!(recv.len(), 18);
    }

    #[tokio::test]
    async fn test_unpublish_entity() {
        let settings = temp_env::with_var_unset("DUMMY", || Settings::new(None).unwrap());
        let event_channel = EventChannel::new();
        let container = HomeAssistantContainer::new(
            Arc::new(settings),
            event_channel.get_sender(),
            &ContainerEventInfo {
                name: "/Container Name".into(),
                is_host_networking: false,
            },
            "node_id".into(),
        );
        let entity = &container.sensors[0];

        let mut recv = event_channel.get_receiver();
        assert_eq!(recv.len(), 0);
        container.unpublish_entity(entity);
        assert_eq!(recv.len(), 1);

        let event = recv.recv().await.unwrap();
        assert!(matches!(event, Event::PublishMqttMessage { .. }));
        if let Event::PublishMqttMessage(msg) = event {
            assert_eq!(msg.payload.len(), 0);
        }
    }

    fn get_container(
        node_id: &str,
        container_name: &str,
        is_host_networking: bool,
    ) -> HomeAssistantContainer {
        let settings = temp_env::with_var_unset("DUMMY", || Settings::new(None).unwrap());
        let event_channel = EventChannel::new();
        HomeAssistantContainer::new(
            Arc::new(settings),
            event_channel.get_sender(),
            &ContainerEventInfo {
                name: container_name.into(),
                is_host_networking,
            },
            node_id.into(),
        )
    }

    fn assert_sensor(
        container: &HomeAssistantContainer,
        name: &str,
        unique_id: &str,
        object_id: &str,
        enabled_by_default: bool,
    ) {
        let sensor = container.sensors.iter().find(|s| s.name == name).unwrap();
        assert_eq!(sensor.unique_id.as_deref(), Some(unique_id));
        assert_eq!(sensor.object_id.as_deref(), Some(object_id));
        assert_eq!(sensor.default_entity_id.as_deref(), Some(object_id));
        assert_eq!(sensor.enabled_by_default, Some(enabled_by_default));
    }

    fn assert_button(
        container: &HomeAssistantContainer,
        name: &str,
        unique_id: &str,
        object_id: &str,
        enabled_by_default: bool,
    ) {
        let button = container.buttons.iter().find(|b| b.name == name).unwrap();
        assert_eq!(button.unique_id.as_deref(), Some(unique_id));
        assert_eq!(button.object_id.as_deref(), Some(object_id));
        assert_eq!(button.default_entity_id.as_deref(), Some(object_id));
        assert_eq!(button.enabled_by_default, Some(enabled_by_default));
    }
}
