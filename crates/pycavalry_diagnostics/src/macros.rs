// This file is part of pycavalry.
//
// pycavalry is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[macro_export]
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

#[macro_export]
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
            pub fn new($($prop: impl Into<$prop_typ>,)* range: TextRange) -> Self {
                Self { $($prop: $prop.into(),)* range }
            }
        }

        pycavalry_diagnostics::impl_diagnostic_to_box!($typ);

        impl std::fmt::Display for $typ {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut temp_vec = Vec::new();
                $(
                    temp_vec.push(format!("{}", self.$prop));
                )*
                write!(f, concat!(stringify!($typ), "({})"), temp_vec.join(", "))
            }
        }

        impl Diag for $typ {
            fn print<'a>(&'a $self, file_name: &'a str) -> DiagReport<'a> {
                use pycavalry_diagnostics;
                let color = pycavalry_diagnostics::type_to_color(&$kind);
                let kind = pycavalry_diagnostics::type_to_kind(&$kind);
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
