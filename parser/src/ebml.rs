use nom::IResult;

use crate::{
    element::Element,
    element_metadata::parse_element_metadata,
    integer::{parse_int, UnsignedElement},
    parser_utils::{check_id_matches, check_input_buffer_is_big_enough},
    string::{parse_string, StringElement},
    ElementMetadata, Id,
};

#[derive(Debug, PartialEq)]
pub struct Ebml {
    pub ebml_version: UnsignedElement,
    pub ebml_read_version: UnsignedElement,
    pub ebml_max_id_length: UnsignedElement,
    pub ebml_max_size_length: UnsignedElement,
    pub doc_type: StringElement,
    pub doc_type_version: UnsignedElement,
    pub doc_type_read_version: UnsignedElement,
}

// #[derive(Debug, PartialEq)]
// pub struct Ebml {
//     pub ebml_version: u64,
//     pub ebml_read_version: u64,
//     pub ebml_max_id_length: u64,
//     pub ebml_max_size_length: u64,
//     pub doc_type: String,
//     pub doc_type_version: u64,
//     pub doc_type_read_version: u64,
// }

pub fn parse_ebml(input: &[u8]) -> IResult<&[u8], Element<Ebml>> {
    let (input, metadata) = parse_element_metadata(input)?;
    check_id_matches(input, metadata.id, Id::Ebml)?;
    check_input_buffer_is_big_enough(input, metadata.size)?;

    let (input, ebml_version) = parse_int::<u64>(input)?;
    let (input, ebml_read_version) = parse_int::<u64>(input)?;
    let (input, ebml_max_id_length) = parse_int::<u64>(input)?;
    let (input, ebml_max_size_length) = parse_int::<u64>(input)?;
    let (input, doc_type) = parse_string(input)?;
    let (input, doc_type_version) = parse_int::<u64>(input)?;
    let (input, doc_type_read_version) = parse_int::<u64>(input)?;

    Ok((
        input,
        Element {
            value: Ebml {
                ebml_version,
                ebml_read_version,
                ebml_max_id_length,
                ebml_max_size_length,
                doc_type,
                doc_type_version,
                doc_type_read_version,
            },
            metadata,
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::Id;

    use super::*;

    #[test]
    fn test_parse_ebml() {
        const INPUT: &[u8] = &[
            0x1a, 0x45, 0xdf, 0xa3, 0x9f, 0x42, 0x86, 0x81, 0x01, 0x42, 0xf7, 0x81, 0x01, 0x42,
            0xf2, 0x81, 0x04, 0x42, 0xf3, 0x81, 0x08, 0x42, 0x82, 0x84, 0x77, 0x65, 0x62, 0x6d,
            0x42, 0x87, 0x81, 0x04, 0x42, 0x85, 0x81, 0x02,
        ];
        const EMPTY: &[u8] = &[];

        let (input, ebml_element) = parse_ebml(INPUT).unwrap();
        assert_eq!(input, EMPTY);

        assert_eq!(
            ebml_element.value(),
            &Ebml {
                ebml_version: UnsignedElement {
                    value: 1,
                    metadata: ElementMetadata {
                        id: Id::EbmlVersion,
                        header_size: 3,
                        size: 1
                    }
                },
                ebml_read_version: UnsignedElement {
                    value: 1,
                    metadata: ElementMetadata {
                        id: Id::EbmlReadVersion,
                        header_size: 3,
                        size: 1
                    }
                },
                ebml_max_id_length: UnsignedElement {
                    value: 4,
                    metadata: ElementMetadata {
                        id: Id::EbmlMaxIdLength,
                        header_size: 3,
                        size: 1
                    }
                },
                ebml_max_size_length: UnsignedElement {
                    value: 8,
                    metadata: ElementMetadata {
                        id: Id::EbmlMaxSizeLength,
                        header_size: 3,
                        size: 1
                    }
                },
                doc_type: StringElement {
                    value: "webm".to_string(),
                    metadata: ElementMetadata {
                        id: Id::DocType,
                        header_size: 3,
                        size: 4
                    }
                },
                doc_type_version: UnsignedElement {
                    value: 4,
                    metadata: ElementMetadata {
                        id: Id::DocTypeVersion,
                        header_size: 3,
                        size: 1
                    }
                },
                doc_type_read_version: UnsignedElement {
                    value: 2,
                    metadata: ElementMetadata {
                        id: Id::DocTypeReadVersion,
                        header_size: 3,
                        size: 1
                    }
                }
            }
        );
    }
}
