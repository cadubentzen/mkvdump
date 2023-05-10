macro_rules! ebml_elements {
    ($($(#[doc = $doc:literal])* name = $element_name:ident, original_name = $original_name:expr, id = $id:expr, variant = $variant:ident;)+) => {
        use serde::{Serialize, Serializer};

        /// Matroska Element Type.
        pub enum Type {
            /// Unsigned
            Unsigned,
            /// Signed
            Signed,
            /// Float
            Float,
            /// String
            String,
            /// Utf8
            Utf8,
            /// Date
            Date,
            /// Master
            Master,
            /// Binary
            Binary,
        }

        /// Matroska Element ID.
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub enum Id {
            /// Unknown ID containing the value parsed.
            Unknown(u32),
            /// Corrupted element. Used when there is a parsing error and a portion of the input is skipped.
            Corrupted,
            $(
                $(#[doc = $doc])*
                $element_name,
            )+
        }

        impl Id {
            /// Build a new ID from an u32. If the value does not represent a known element,
            /// an Unknown ID will be created.
            pub fn new(id: u32) -> Self {
                match id {
                    $($id => Self::$element_name,)+
                    _ => Self::Unknown(id)
                }
            }

            /// Build a special corrupted ID
            pub fn corrupted() -> Self {
                Self::Corrupted
            }

            /// Get type of element for this ID
            pub fn get_type(&self) -> Type {
                match self {
                    $(Id::$element_name => Type::$variant,)+
                    Id::Unknown(_) | Id::Corrupted => Type::Binary
                }
            }

            /// Get underlying integer value
            pub fn get_value(&self) -> Option<u32> {
                match self {
                    $(Id::$element_name => Some($id),)+
                    Id::Unknown(value) => Some(*value),
                    Id::Corrupted => None
                }
            }
        }

        impl Serialize for Id {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                match *self {
                    $(Id::$element_name => s.serialize_str($original_name),)+
                    Id::Unknown(value) => s.serialize_str(&format!("0x{:X}", value)),
                    Id::Corrupted => s.serialize_str("Corrupted")
                }
            }
        }
    };
}

macro_rules! ebml_enumerations {
    ($($id:ident { $($(#[doc = $doc:expr])* $variant:ident = $value:expr, original_label = $original_label:expr;)+ };)+) => {
        use crate::elements::Id;
        use serde::Serialize;

        $(
            #[derive(Debug, PartialEq, Eq, Clone, Serialize)]
            pub enum $id {
                $(
                    $(#[doc = $doc])*
                    #[serde(rename = $original_label)]
                    $variant,
                )+
            }

            impl $id {
                pub fn new(value: u64) -> Option<Self> {
                    match value {
                        $($value => Some(Self::$variant),)+
                        _ => None,
                    }
                }
            }
        )+

        /// Enumeration of values for a given Matroska Element.
        #[derive(Debug, PartialEq, Eq, Clone, Serialize)]
        #[serde(untagged)]
        pub enum Enumeration {
            /// Uknown variant, which simply carries the value.
            Unknown(u64),
            $($id($id),)+
        }

        impl Enumeration {
            /// Create new enumeration
            pub fn new(id: &Id, value: u64) -> Self {
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
