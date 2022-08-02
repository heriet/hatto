use crate::cyclonedx::{Component, Components};
use crate::error::Error;

use minidom::Element;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize, Debug, PartialEq)]
pub enum BomFormat {
    CycloneDX,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Bom {
    pub bom_format: BomFormat,
    pub spec_version: String,
    pub serial_number: Option<String>,
    pub version: u32,
    pub components: Option<Components>,
    // other field is umimplemented
}

pub fn load_json_file(source: &File) -> Result<Bom, Error> {
    let reader = BufReader::new(source);
    let bom = serde_json::from_reader(reader)?;

    Ok(bom)
}

pub fn load_xml_file(source: &File) -> Result<Bom, Error> {
    let reader = BufReader::new(source);
    let element = Element::from_reader(reader)?;

    let mut bom = Bom {
        bom_format: BomFormat::CycloneDX,
        spec_version: "".to_string(),
        serial_number: None,
        version: 0,
        components: None,
    };

    if let Some(serial_number) = element.attr("serialNumber") {
        bom.serial_number = Some(serial_number.to_string());
    }

    if let Some(version) = element.attr("version") {
        bom.version = version.parse::<u32>()?;
    }

    for child in element.children() {
        if child.name() == "components" {
            bom.components = Some(Components::from(
                child
                    .children()
                    .map(Component::from)
                    .collect::<Vec<_>>(),
            ));
        }
    }

    Ok(bom)
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn load_json_file_valid() {
        let file = File::open("test/cyclonedx/basic.cdx.json").unwrap();
        let bom: Bom = load_json_file(&file).unwrap();

        assert_eq!(bom.bom_format, BomFormat::CycloneDX);
    }

    #[test]
    fn load_xml_file_valid() {
        let file = File::open("test/cyclonedx/basic.cdx.xml").unwrap();
        let bom: Bom = load_xml_file(&file).unwrap();

        assert_eq!(bom.bom_format, BomFormat::CycloneDX);
    }
}
