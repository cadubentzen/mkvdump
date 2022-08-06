// ebml_elements! {
//     name = EBML, id = 0x1A45DFA3, type = master;
//     name = EBMLVersion, id = 0x4286, type = uinteger;
//     name = EBMLReadVersion, id = 0x42F7, type = uinteger;
//     name = EBMLMaxIDLength, id = 0x42F2, type = uinteger;
//     name = EBMLMaxSizeLength, id = 0x42F3, type = uinteger;
// }

macro_rules! ebml_elements {
    ($(name = $element_name:ident, id = $id:expr, variant = $variant:ident;)+) => {
        use serde::Serialize;

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

        #[derive(Debug, PartialEq, Clone, Serialize)]
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


    };
}

pub(crate) use ebml_elements;
