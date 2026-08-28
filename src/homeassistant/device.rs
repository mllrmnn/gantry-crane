use serde::Serialize;

use super::entity::normalize_unique_id_part;

#[derive(Debug, Serialize)]
pub struct Device {
    pub name: String,
    pub identifiers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sw_version: Option<String>,
}

impl Device {
    pub fn new(name: String, manufacturer: Option<String>) -> Self {
        let id = format!("gc_{}", normalize_unique_id_part(&name));
        Self::new_with_identifier(name, id, manufacturer)
    }

    pub fn new_with_identifier(
        name: String,
        identifier: String,
        manufacturer: Option<String>,
    ) -> Self {
        Self {
            name,
            identifiers: vec![identifier],
            manufacturer,
            model: None,
            sw_version: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Device;

    #[test]
    fn test_new() {
        let name = "Test Device".to_owned();
        let manufacturer = Some("Test Manufacturer".into());
        let dev = Device::new(name.clone(), manufacturer.clone());

        assert_eq!(dev.name, name);
        assert_eq!(dev.manufacturer, manufacturer);
        assert_eq!(dev.identifiers.len(), 1);
        assert!(!dev.identifiers[0].contains(" "));
    }

    #[test]
    fn test_new_with_identifier() {
        let dev = Device::new_with_identifier(
            "dockerproxy (docker01-rrnuc)".into(),
            "gc_docker01-rrnuc_dockerproxy".into(),
            Some("Docker".into()),
        );

        assert_eq!(dev.name, "dockerproxy (docker01-rrnuc)");
        assert_eq!(
            dev.identifiers,
            vec!["gc_docker01-rrnuc_dockerproxy".to_owned()]
        );
    }
}
