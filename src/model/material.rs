use crate::cli::SourceType;
use crate::cyclonedx::{load_json_file, load_xml_file, Bom, Components, LicenseChoice, Licenses};
use crate::error::Error;

use csv::ReaderBuilder;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use spdx_rs::models::{SPDX, SpdxExpression};
use spdx_rs::parsers::spdx_from_tag_value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};

#[derive(Debug, Serialize, Clone)]
#[pyclass]
pub struct Material {
    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get, set)]
    pub version: Option<String>,

    #[pyo3(get, set)]
    pub licenses: Vec<String>,

    #[pyo3(get, set)]
    pub annotations: HashMap<String, String>,
}

#[pymethods]
impl Material {
    fn update_annotation(&mut self, key: String, value: String) {
        self.annotations.insert(key, value);
    }
}


#[derive(Debug, Deserialize)]
struct SourceTsv {
    name: String,
    version: String,
    licenses: String,
    annotations: String,
}

pub fn load_materials(source: &File, source_type: &SourceType) -> Result<Vec<Material>, Error> {
    match source_type {
        SourceType::Tsv => load_materials_tsv(source),
        SourceType::SpdxTag => load_materials_spdx_tag(source),
        SourceType::SpdxJson => load_materials_spdx_json(source),
        SourceType::SpdxYaml => load_materials_spdx_json(source), // yaml parser equals json parser
        SourceType::CycloneDxJson => load_materials_cyclonedx_json(source),
        SourceType::CycloneDxXml => load_materials_cyclonedx_xml(source),
    }
}

fn load_materials_tsv(source: &File) -> Result<Vec<Material>, Error> {
    let mut materials: Vec<Material> = Vec::new();

    let mut tsv_reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b'\t')
        .comment(Some(b'#'))
        .from_reader(source);
    for row in tsv_reader.deserialize() {
        let tsv: SourceTsv = row?;
        let annotations = tsv
            .annotations
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|kv| kv.split('=').map(|s| s.to_string()))
            .map(|mut kv| -> (String, String) { 
                (kv.next().unwrap_or_else(|| "".to_string()), kv.next().unwrap_or_else(|| "".to_string())) 
            })
            .collect::<HashMap<String, String>>();
        let material = Material {
            name: tsv.name,
            version: Some(tsv.version),
            licenses: tsv.licenses.split(',').map(|s| s.to_string()).collect(),
            annotations,
        };
        materials.push(material);
    }

    Ok(materials)
}

fn load_materials_spdx_tag(source: &File) -> Result<Vec<Material>, Error> {
    let mut content = String::new();
    let mut reader = BufReader::new(source);
    reader.read_to_string(&mut content)?;
    let spdx = spdx_from_tag_value(&content)?;

    extract_spdx(&spdx)
}

fn load_materials_spdx_json(source: &File) -> Result<Vec<Material>, Error> {
    let spdx: SPDX = serde_json::from_reader(source)?;

    extract_spdx(&spdx)
}

fn extract_spdx(spdx: &SPDX) -> Result<Vec<Material>, Error> {
    let mut materials: Vec<Material> = Vec::new();
    for pi in &spdx.package_information {
        let concluded_licenses = pi
            .concluded_license
            .licenses()
            .iter()
            .map(|&license| license.identifier.clone())
            .collect::<Vec<_>>();

        let licenses = if concluded_licenses.len() == 1 && concluded_licenses[0] == "NONE" {
            pi.all_licenses_information_from_files.to_vec()
        } else if concluded_licenses.is_empty() {
            concluded_licenses 
        } else {
            pi.all_licenses_information_from_files.to_vec()
        };

        let material = Material {
            name: pi.package_name.clone(),
            version: pi.package_version.clone(),
            licenses,
            annotations: HashMap::new(),
        };
        materials.push(material);
    }

    Ok(materials)
}

fn load_materials_cyclonedx_json(source: &File) -> Result<Vec<Material>, Error> {
    let bom = load_json_file(source)?;

    extract_cyclonedx(&bom)
}

fn load_materials_cyclonedx_xml(source: &File) -> Result<Vec<Material>, Error> {
    let bom = load_xml_file(source)?;

    extract_cyclonedx(&bom)
}

fn extract_cyclonedx(bom: &Bom) -> Result<Vec<Material>, Error> {
    let mut materials: Vec<Material> = Vec::new();

    if let Some(components) = &bom.components {
        extract_cyclonedx_components(components, &mut materials)?
    }

    Ok(materials)
}

fn extract_cyclonedx_components(components: &Components, materials: &mut Vec<Material>) -> Result<(), Error> {
    for component in &components.0 {
        let licenses = match &component.licenses {
            Some(l) => extract_cyclonedx_licenses(l)?,
            None => Vec::new(),
        };

        let material = Material {
            name: component.name.clone(),
            version: component.version.clone(),
            licenses,
            annotations: HashMap::new(),
        };

        materials.push(material);

        if let Some(components) = &component.components {
            extract_cyclonedx_components(components, materials)?;
        }
    }

    Ok(())
}

fn extract_cyclonedx_licenses(licenses: &Licenses) -> Result<Vec<String>, Error> {
    let mut lis: Vec<String> = Vec::new();

    for license_choice in &licenses.0 {
        match license_choice {
            LicenseChoice::License(license) => {
                if license.id != None {
                    if let Some(id) = &license.id {
                        lis.push(id.clone());
                    }
                } else if license.name != None {
                    if let Some(name) = &license.name {
                        lis.push(name.clone());
                    }
                }
            },
            LicenseChoice::Expression(expression) => {
                let spdx_expression = SpdxExpression::parse(&expression)?;
                lis.append(&mut spdx_expression.identifiers().iter().map(|s| s.to_string()).collect::<Vec<String>>());
            }
        }
    }

    Ok(lis)
}
