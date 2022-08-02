use minidom::Element;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Licenses(pub Vec<LicenseChoice>);

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LicenseChoice {
    License(License),
    Expression(String),
    // other field is umimplemented
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub id: Option<String>,
    pub name: Option<String>,
}

impl From<Vec<LicenseChoice>> for Licenses {
    fn from(vec: Vec<LicenseChoice>) -> Self {
        Licenses(vec)
    }
}

impl From<&Element> for LicenseChoice {
    fn from(element: &Element) -> Self {
        match element.name() {
            "license" => LicenseChoice::License(License::from(element)),
            "expression" => LicenseChoice::Expression(element.text()),
            _ => LicenseChoice::Expression("".to_string()),
        }
    }
}

impl From<&Element> for License {
    fn from(element: &Element) -> Self {
        let mut license = License {
            id: None,
            name: None,
        };

        for child in element.children() {
            match child.name() {
                "id" => {
                    license.id = Some(child.text());
                }
                "name" => {
                    license.name = Some(child.text());
                }
                _ => (),
            }
        }

        license
    }
}
