macro_rules! impl_diagnostic_to_box {
    ( $typ:ident ) => {
        impl From<$typ> for Box<dyn Diag> {
            fn from(val: $typ) -> Self {
                Box::new(val)
            }
        }
    };
}
pub(crate) use impl_diagnostic_to_box;

macro_rules! custom_diagnostic {
    ( ($typ:ident, $self:ident, $kind:expr), ($( $prop:ident: $prop_typ:ty ),*), $func:expr ) => {
        #[derive(Debug, PartialEq)]
        pub struct $typ {
            $(
                pub $prop: $prop_typ,
            )*
            pub range: TextRange,
        }

        impl $typ {
            pub fn new($($prop: $prop_typ,)* range: TextRange) -> Self {
                Self { $($prop,)* range }
            }
        }

        crate::diagnostics::macros::impl_diagnostic_to_box!($typ);

        impl Diag for $typ {
            fn print<'a>(&'a $self, file_name: &'a str) -> DiagReport<'a> {
                use crate::diagnostics::{type_to_color, type_to_kind};
                let color = type_to_color(&$kind);
                let kind = type_to_kind(&$kind);
                Report::build(kind, file_name, $self.range.start().to_usize())
                    .with_label(
                        Label::new((file_name, convert_range($self.range)))
                            .with_message($func($self, color))
                            .with_color(color),
                    )
                    .finish()
            }
        }
    };
}
pub(crate) use custom_diagnostic;
