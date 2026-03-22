macro_rules! count_variants {
    ($($variant:ident),+ $(,)?) => {
        <[()]>::len(&[$($crate::enum_meta::count_variants!(@unit $variant)),+])
    };
    (@unit $variant:ident) => {
        ()
    };
}

macro_rules! define_key_enum {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $key:literal
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),+
        }

        impl $name {
            #[allow(dead_code)]
            pub const ALL: [Self; $crate::enum_meta::count_variants!($($variant),+)] =
                [$(Self::$variant),+];

            pub const fn key(self) -> &'static str {
                match self {
                    $(Self::$variant => $key),+
                }
            }

            pub fn from_key(raw: &str) -> Option<Self> {
                match raw {
                    $($key => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

macro_rules! define_labeled_key_enum {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => { key: $key:literal, label: $label:literal }
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),+
        }

        impl $name {
            #[allow(dead_code)]
            pub const ALL: [Self; $crate::enum_meta::count_variants!($($variant),+)] =
                [$(Self::$variant),+];

            pub const fn key(self) -> &'static str {
                match self {
                    $(Self::$variant => $key),+
                }
            }

            pub const fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label),+
                }
            }

            pub fn from_key(raw: &str) -> Option<Self> {
                match raw {
                    $($key => Some(Self::$variant),)+
                    _ => None,
                }
            }

            #[allow(dead_code)]
            pub fn from_label(raw: &str) -> Option<Self> {
                $(
                    if raw == $label {
                        return Some(Self::$variant);
                    }
                )+
                None
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.label())
            }
        }
    };
}

pub(crate) use count_variants;
pub(crate) use define_key_enum;
pub(crate) use define_labeled_key_enum;
