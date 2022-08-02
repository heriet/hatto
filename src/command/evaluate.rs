use crate::cli::{EvaluateArgs, OutputFormat, SourceType};
use crate::error::Error;
use crate::model::material::{load_materials, Material};

use ansi_term::Color::{Green, Red, Yellow};
use anyhow::{bail, Result};
use pyo3::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Serialize)]
struct MaterialResult {
    pub material: Material,
    pub result: EvaluateResult,
}

#[derive(Debug, Serialize, Clone)]
#[pyclass]
struct EvaluateResult {
    #[pyo3(get)]
    pub success: bool,

    #[pyo3(get)]
    pub errors: Vec<String>,

    #[pyo3(get)]
    pub warnings: Vec<String>,
}

#[pymethods]
impl EvaluateResult {
    #[new]
    fn new() -> Self {
        EvaluateResult {
            success: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_error(&mut self, message: String) {
        self.errors.push(message);
        self.success = false;
    }

    fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }
}

const DEFAULT_POLICY: &str = r#"
#!/usr/bin/python

allowed_licenses = [
    "Apache-2.0",
    "MIT",
    "BSD-3-Clause",
    "Unlicense",
]

def evaluate(material, result):
    for license in material.licenses:
        if license not in allowed_licenses:
           result.add_error(f"{license} is not allowed")
"#;

const DEFAULT_CURATION: &str = r#"
#!/usr/bin/python

def curate_material(material):
    pass
"#;

pub fn exec(args: &EvaluateArgs) -> Result<()> {
    let policy_py = load_policy(args)?;
    let curation_py = load_curation(args)?;

    let source_file = match File::open(&args.source) {
        Err(err) => bail!(Error::Io(err)),
        Ok(file) => file,
    };

    let source_type = match &args.source_type {
        Some(s) => s.clone(),
        None => detect_source_type(&args.source),
    };

    let materials: Vec<Material> = load_materials(&source_file, &source_type)?;
    let mut results: Vec<MaterialResult> = Vec::new();
    let mut success = true;

    let py_result: PyResult<()> = Python::with_gil(|py| {
        let policy_module = PyModule::from_code(py, &policy_py, "", "")?;
        let evaluate: Py<PyAny> = policy_module.getattr("evaluate")?.into();

        let curation_module = PyModule::from_code(py, &curation_py, "", "")?;
        let curate: Py<PyAny> = curation_module.getattr("curate_material")?.into();

        for material in materials {
            // call curate
            let py_material = Py::new(py, material.clone())?;
            curate.call1(py, (&py_material,))?;
            let curated_material: Material = py_material.extract(py)?;

            // call evaluate
            let py_curated_material = Py::new(py, curated_material.clone())?;
            let py_evaluate_result = Py::new(py, EvaluateResult::new())?;
            evaluate.call1(py, (&py_curated_material, &py_evaluate_result))?;
            let evaluate_result: EvaluateResult = py_evaluate_result.extract(py)?;

            if args.output_format == OutputFormat::Human {
                print_evaluate_result_for_human(&curated_material, &evaluate_result);
            }

            if !evaluate_result.success {
                success = false;
            }

            results.push(MaterialResult {
                material: curated_material,
                result: evaluate_result,
            });
        }
        Ok(())
    });

    if let Err(err) = py_result {
        bail!(err)
    }

    if args.output_format == OutputFormat::Json {
        println!("{}", json!(results));
    }

    match success {
        true => Ok(()),
        false => bail!(Error::Failure("evaluate failed".to_string())),
    }
}

fn print_evaluate_result_for_human(material: &Material, result: &EvaluateResult) {
    let version = material.version.clone().unwrap_or_else(|| "".to_string());

    if result.success {
        println!(
            "{} {} {} licenses:{:?} annotations:{:?}",
            Green.paint("OK"),
            material.name,
            version,
            material.licenses,
            material.annotations
        );
    } else {
        println!(
            "{} {} {} licenses:{:?} annotations:{:?}",
            Red.paint("NG"),
            material.name,
            version,
            material.licenses,
            material.annotations
        );
    }

    let indent = "  "; // two spaces

    for message in &result.errors {
        println!("{}{} {}", indent, Red.paint("ERROR"), message);
    }

    for message in &result.warnings {
        println!("{}{} {}", indent, Yellow.paint("WARNING"), message);
    }
}

fn detect_source_type<P: AsRef<Path>>(path: &P) -> SourceType {
    let p_ref = path.as_ref();
    let path_str = p_ref.to_str().unwrap();

    if path_str.ends_with(".tsv") {
        return SourceType::Tsv;
    } else if path_str.ends_with(".spdx") {
        return SourceType::SpdxTag;
    } else if path_str.ends_with(".spdx.json") {
        return SourceType::SpdxJson;
    } else if path_str.ends_with(".spdx.yml") || path_str.ends_with(".spdx.yaml") {
        return SourceType::SpdxYaml;
    } else if p_ref.ends_with("bom.json") || path_str.ends_with(".cdx.json") {
        return SourceType::CycloneDxJson;
    } else if p_ref.ends_with("bom.xml") || path_str.ends_with(".cdx.xml") {
        return SourceType::CycloneDxXml;
    }

    SourceType::Tsv
}

fn load_policy(args: &EvaluateArgs) -> Result<String, Error> {
    let policy_path = match &args.policy {
        Some(v) => v,
        None => return Ok(DEFAULT_POLICY.to_string()),
    };

    let mut policy_file = match File::open(policy_path) {
        Err(err) => return Err(Error::Io(err)),
        Ok(file) => file,
    };

    let mut content = String::new();
    match policy_file.read_to_string(&mut content) {
        Err(err) => return Err(Error::Io(err)),
        Ok(_) => return Ok(content),
    };
}

fn load_curation(args: &EvaluateArgs) -> Result<String, Error> {
    let curation_path = match &args.curation {
        Some(v) => v,
        None => return Ok(DEFAULT_CURATION.to_string()),
    };

    let mut curation_file = match File::open(curation_path) {
        Err(err) => return Err(Error::Io(err)),
        Ok(file) => file,
    };

    let mut content = String::new();
    match curation_file.read_to_string(&mut content) {
        Err(err) => return Err(Error::Io(err)),
        Ok(_) => return Ok(content),
    };
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_source_type() {
        let path_tsv = Path::new("./foo/bar.tsv");
        assert_eq!(detect_source_type(&path_tsv), SourceType::Tsv);

        let path_spdx = Path::new("./foo/bar.spdx");
        assert_eq!(detect_source_type(&path_spdx), SourceType::SpdxTag);

        let path_spdx_json = Path::new("./foo/bar.spdx.json");
        assert_eq!(detect_source_type(&path_spdx_json), SourceType::SpdxJson);

        let path_spdx_yml = Path::new("./foo/bar.spdx.yml");
        assert_eq!(detect_source_type(&path_spdx_yml), SourceType::SpdxYaml);

        let path_spdx_yaml = Path::new("./foo/bar.spdx.yaml");
        assert_eq!(detect_source_type(&path_spdx_yaml), SourceType::SpdxYaml);

        let path_cdx_json = Path::new("./foo/bar.cdx.json");
        assert_eq!(
            detect_source_type(&path_cdx_json),
            SourceType::CycloneDxJson
        );

        let path_bom_json = Path::new("./foo/bom.json");
        assert_eq!(
            detect_source_type(&path_bom_json),
            SourceType::CycloneDxJson
        );

        let path_cdx_xml = Path::new("./foo/bar.cdx.xml");
        assert_eq!(detect_source_type(&path_cdx_xml), SourceType::CycloneDxXml);

        let path_bom_xml = Path::new("./foo/bom.xml");
        assert_eq!(detect_source_type(&path_bom_xml), SourceType::CycloneDxXml);
    }
}
