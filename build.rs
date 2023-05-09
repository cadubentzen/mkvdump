use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{collections::HashSet, env};

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

const EBML_XML: &str = include_str!("ebml.xml");
const EBML_MATROSKA_XML: &str = include_str!("ebml_matroska.xml");

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct EBMLSchema {
    #[serde(rename(deserialize = "$value"))]
    elements: Vec<Element>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Element {
    name: String,
    path: String,
    id: String,
    #[serde(rename(deserialize = "type"))]
    variant: String,
    #[serde(rename(deserialize = "$value"))]
    details: Option<Vec<ElementDetail>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ElementDetail {
    Documentation(Documentation),
    Extension(Extension),
    Restriction(Restriction),
    ImplementationNote(ImplementationNote),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Documentation {
    #[serde(rename(deserialize = "$value"))]
    text: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Extension {
    webm: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Restriction {
    #[serde(rename(deserialize = "$value"))]
    enums: Vec<Enum>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Enum {
    value: String,
    label: String,
    #[serde(rename(deserialize = "$value"))]
    documentation: Option<Vec<Documentation>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct ImplementationNote {
    #[serde(rename(deserialize = "$value"))]
    text: String,
}

fn get_elements() -> Vec<Element> {
    let ebml_schema: EBMLSchema = serde_xml_rs::from_str(EBML_XML).unwrap();
    let ebml_matroska_schema: EBMLSchema = serde_xml_rs::from_str(EBML_MATROSKA_XML).unwrap();

    // Ignoring Matroska overrides of EBML elements
    let mut known_elements = HashSet::<String>::new();
    let mut elements = Vec::<Element>::new();
    for element in ebml_schema
        .elements
        .into_iter()
        .chain(ebml_matroska_schema.elements.into_iter())
    {
        if known_elements.get(&element.name).is_none() {
            known_elements.insert(element.name.clone());
            elements.push(element);
        }
    }

    // Pre-format names and variants
    elements.iter_mut().for_each(|e| {
        e.variant = variant_to_enum_literal(&e.variant).to_string();
    });

    elements
}

fn variant_to_enum_literal(variant: &str) -> &str {
    match variant {
        "master" => "Master",
        "uinteger" => "Unsigned",
        "integer" => "Signed",
        "string" => "String",
        "binary" => "Binary",
        "utf-8" => "Utf8",
        "date" => "Date",
        "float" => "Float",
        _ => panic!("Variant not expected: {}", variant),
    }
}

fn apply_label_quirks(label: &str, reserved_index: &mut i32) -> String {
    let mut label = label
        .replace(|c: char| !c.is_ascii_alphanumeric(), " ")
        .to_case(Case::Pascal);

    // Hack because identifiers can't start with a number
    if label == "3Des" {
        label = "TripleDes".to_string();
    }
    // "Reserved" sometimes repeats in enums
    else if label == "Reserved" {
        label = format!("Reserved{}", reserved_index);
        *reserved_index += 1;
    }

    label
}

fn create_elements_file(elements: &[Element]) -> std::io::Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let elements_path = Path::new(&out_dir).join("elements.rs");
    let mut file = File::create(elements_path)?;

    writeln!(file, "use crate::ebml::ebml_elements;")?;
    writeln!(file, "ebml_elements! {{")?;

    for Element {
        name,
        id,
        variant,
        path: _,
        details,
    } in elements
    {
        if let Some(details) = details {
            macro_rules! write_comment_lines {
                ($detail_type:path) => {
                    for detail in details {
                        if let $detail_type(detail) = detail {
                            detail.text.split('\n').filter(|s| !s.is_empty()).for_each(
                                |doc_line| writeln!(file, "    /// {}", doc_line).unwrap(),
                            );
                        }
                    }
                };
            }

            write_comment_lines!(ElementDetail::Documentation);
            write_comment_lines!(ElementDetail::ImplementationNote);
        }

        let enum_name = name.to_case(Case::Pascal);
        writeln!(
            file,
            "    name = {enum_name}, original_name = \"{name}\", id = {id}, variant = {variant};"
        )?;
    }
    writeln!(file, "}}")?;

    Ok(())
}

fn create_enumerations_file(elements: &[Element]) -> std::io::Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let enumerations_path = Path::new(&out_dir).join("enumerations.rs");
    let mut file = File::create(enumerations_path)?;

    writeln!(file, "use crate::ebml::ebml_enumerations;")?;
    writeln!(file, "ebml_enumerations! {{")?;

    for element in elements {
        let mut reserved_index = 1;
        if element.variant != "Unsigned" {
            continue;
        }
        let enum_name = element.name.to_case(Case::Pascal);
        if let Some(details) = &element.details {
            for detail in details {
                if let ElementDetail::Restriction(restriction) = detail {
                    writeln!(file, "    {} {{", enum_name)?;
                    for enumeration in &restriction.enums {
                        if let Some(docs) = &enumeration.documentation {
                            for doc in docs {
                                doc.text.split('\n').filter(|s| !s.is_empty()).for_each(
                                    |doc_line| writeln!(file, "        /// {}", doc_line).unwrap(),
                                );
                            }
                        } else {
                            writeln!(file, "        /// {}", enumeration.label)?;
                        }
                        let label = apply_label_quirks(&enumeration.label, &mut reserved_index);
                        writeln!(
                            file,
                            "        {} = {}, original_label = \"{}\";",
                            label, enumeration.value, enumeration.label
                        )?;
                    }
                    writeln!(file, "    }};")?;
                }
            }
        }
    }
    writeln!(file, "}}")?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=ebml.xml");
    println!("cargo:rerun-if-changed=ebml_matroska.xml");

    let elements = get_elements();
    create_elements_file(&elements)?;
    create_enumerations_file(&elements)?;

    Ok(())
}
