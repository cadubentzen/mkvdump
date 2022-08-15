macro_rules! ebml_elements {
    ($(name = $element_name:ident, original_name = $original_name:expr, id = $id:expr, variant = $variant:ident;)+) => {
        use serde::{Serialize, Serializer};

        #[derive(Debug, PartialEq)]
        pub(crate) enum Type {
            Unsigned,
            Signed,
            Float,
            String,
            Utf8,
            Date,
            Master,
            Binary,
        }

        #[derive(Debug, PartialEq, Clone)]
        pub(crate) enum Id {
            Unknown(u32),
            $($element_name,)+
        }

        impl Id {
            pub(crate) fn new(id: u32) -> Self {
                match id {
                    $($id => Self::$element_name,)+
                    _ => Self::Unknown(id)
                }
            }

            pub(crate) fn get_type(&self) -> Type {
                match self {
                    $(Id::$element_name => Type::$variant,)+
                    Id::Unknown(_) => Type::Binary
                }
            }
        }

        impl Serialize for Id {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                match *self {
                    $(Id::$element_name => s.serialize_str($original_name),)+
                    Id::Unknown(value) => s.serialize_str(&format!("Unknown (0x{:X})", value))
                }
            }
        }
    };
}

macro_rules! ebml_enumerations {
    ($($id:ident { $($variant:ident = $value:expr, original_label = $original_label:expr;)+ };)+) => {
        use crate::elements::Id;
        use serde::Serialize;

        $(
            #[derive(Debug, PartialEq, Clone, Serialize)]
            pub(crate) enum $id {
                $(
                    #[serde(rename = $original_label)]
                    $variant,
                )+
            }

            impl $id {
                pub(crate) fn new(value: u64) -> Option<Self> {
                    match value {
                        $($value => Some(Self::$variant),)+
                        _ => None,
                    }
                }
            }
        )+

        #[derive(Debug, PartialEq, Clone, Serialize)]
        #[serde(untagged)]
        pub(crate) enum Enumeration {
            Unknown(u64),
            $($id($id),)+
        }

        impl Enumeration {
            pub(crate) fn new(id: &Id, value: u64) -> Self {
                match id {
                    $(
                        Id::$id => $id::new(value).map_or(Self::Unknown(value), Self::$id),
                    )+
                    _ => Self::Unknown(value)
                }
            }
        }

        impl From<u64> for Enumeration {
            fn from(value: u64) -> Self {
                Self::Unknown(value)
            }
        }
    };
}

pub(crate) use ebml_elements;
pub(crate) use ebml_enumerations;
