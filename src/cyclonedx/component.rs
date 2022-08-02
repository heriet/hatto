use crate::cyclonedx::{LicenseChoice, Licenses};

use minidom::Element;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Components(pub Vec<Component>);

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    pub version: Option<String>,
    pub licenses: Option<Licenses>,
    pub components: Option<Components>,
    // other field is umimplemented
}

impl From<Vec<Component>> for Components {
    fn from(vec: Vec<Component>) -> Self {
        Components(vec)
    }
}

impl From<&Element> for Component {
    fn from(element: &Element) -> Self {
        let mut component = Component {
            component_type: "".to_string(),
            name: "".to_string(),
            version: None,
            licenses: None,
            components: None,
        };

        if let Some(component_type) = element.attr("type") {
            component.component_type = component_type.to_string();
        }

        for child in element.children() {
            match child.name() {
                "name" => {
                    component.name = child.text();
                }
                "version" => {
                    component.version = Some(child.text());
                }
                "licenses" => {
                    component.licenses = Some(Licenses::from(
                        child
                            .children()
                            .map(LicenseChoice::from)
                            .collect::<Vec<_>>(),
                    ));
                }
                "components" => {
                    component.components = Some(Components::from(
                        child
                            .children()
                            .map(Component::from)
                            .collect::<Vec<_>>(),
                    ));
                }
                _ => (),
            }
        }

        component
    }
}
