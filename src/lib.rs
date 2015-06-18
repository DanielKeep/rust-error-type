/*!

This crate provides the `error_type!` macro, which is designed to produce a fresh, reasonably well-featured error type from a concise definition.

A simple example of the usage is given below:

```rust
#[macro_use] extern crate error_type;

use std::borrow::Cow;
use std::error::Error;
use std::io;

error_type! {
    #[derive(Debug)]
    pub enum LibError {
        Io(std::io::Error) {
            cause;
        },
        Kaboom(Kaboom) {
            disp (e, fmt) write!(fmt, "{:?}", e);
            desc (_e) "kaboom!";
        },
        Message(Cow<'static, str>) {
            desc (e) &**e;
            from (s: &'static str) s.into();
            from (s: String) s.into();
        },
        Other(Box<Error>) {
            desc (e) e.description();
            cause (e) Some(&**e);
        }
    }
}

#[derive(Debug)]
pub struct Kaboom;
# fn main() {}
```

The expansion of the above includes the following:

- The `LibError` enumeration (plus the provided `#[derive(Debug)]` annotation), with `Io`, `Message` and `Other` variants.

- An implicit `impl From<Payload> for LibError` for each variant's payload type.

- An implicit `impl Display for LibError`, using the existing `Display` implementation for each variant's payload type.

- An implicit `impl Error for LibError`.

- For the `Io` variant:

  - An implicit `description`, forwarded to the existing definition for `std::io::Error`.

  - An automatic `cause`, forwarded to the existing definition for `std::io::Error`.

    **Note**: the automatic `cause` returns the result of `std::io::Error::cause`, *not* the payload itself.  This macro considers the payload to *be* the error, not the underlying cause.

- For the `Kaboom` variant:

  - An explicit `Display` override, since `Kaboom` does not, itself, implement it.

  - An explicit `description`, which just returns a string literal.

- For the `Message` variant:

  - An explicit `description`, which just returns the contents of the `Cow<'static, str>`.

  - An implicit `cause`, which just returns `None`.

  - An explicit `From<&'static str>` conversion.

  - An explicit `From<String>` conversion.

- For the `Other` variant:

  - An explicit `description` which forwards to the existing definition for the boxed `Error`.

  - An explicit `cause` which returns the boxed error *itself* as the cause.  This is distinct from the behaviour of an *automatic* `cause`.

## FAQ

* *Can I use unitary variants; ones without a payload?*

  No, not as yet.  Maybe if there's demand.

* *Can I use tuple variants with more than one element?*

  No.  This would likely be rather inconvenient to implement, due to the way the various parts of the implementation are constructed.  Not impossible, though.

* *Can I use struct variants?*

  No, for much the same reason as tuple variants.

* *Can I have fields common to all variants; i.e. have the enum wrapped in a struct?*

  No.  It would be nice, but I'm not sure how to go about that.  You can always use the expansion of `error_type!` in a custom structure for the added information.

*/

#[doc(hidden)]
#[macro_export]
macro_rules! error_type_as_item {
    ($i:item) => {$i};
}

#[doc(hidden)]
#[macro_export]
macro_rules! error_type_var_body_emit {
    /*
    Nothing left.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident
    ) => {
        // Done.
    };

    /*
    disp () clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        disp ()
        $($tail:tt)*
    ) => {
        impl<'a> $edi_tr for (&'a $err_name, &'a $var_ty) {
            fn error_fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
                ::std::fmt::Display::fmt(self.1, fmt)
            }
        }

        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr $($tail)*
        }
    };

    /*
    disp ((arg, fmt) expr) clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        disp (($disp_arg:ident, $disp_fmt:ident) $disp_expr:expr)
        $($tail:tt)*
    ) => {
        impl<'a> $edi_tr for (&'a $err_name, &'a $var_ty) {
            fn error_fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
                let $disp_arg = self.1;
                let $disp_fmt = fmt;
                $disp_expr
            }
        }

        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr $($tail)*
        }
    };

    /*
    desc () clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        desc ()
        $($tail:tt)*
    ) => {
        impl<'a> $ede_tr<'a> for (&'a $err_name, &'a $var_ty) {
            fn error_desc(&self) -> &'a str {
                ::std::error::Error::description(self.1)
            }
        }

        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr $($tail)*
        }
    };

    /*
    desc ((arg) expr) clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        desc (($desc_arg:ident) $desc_expr:expr)
        $($tail:tt)*
    ) => {
        impl<'a> $ede_tr<'a> for (&'a $err_name, &'a $var_ty) {
            fn error_desc(&self) -> &'a str {
                let $desc_arg = self.1;
                $desc_expr
            }
        }
        
        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr $($tail)*
        }
    };

    /*
    cause () clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        cause ()
        $($tail:tt)*
    ) => {
        impl<'a> $ec_tr<'a> for (&'a $err_name, &'a $var_ty) {
            fn error_cause(&self) -> ::std::option::Option<&'a ::std::error::Error> {
                None
            }
        }

        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr $($tail)*
        }
    };

    /*
    cause ((arg) expr) clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        cause (($cl_arg:ident) $cl_expr:expr)
        $($tail:tt)*
    ) => {
        impl<'a> $ec_tr<'a> for (&'a $err_name, &'a $var_ty) {
            fn error_cause(&self) -> ::std::option::Option<&'a ::std::error::Error> {
                let $cl_arg = self.1;
                $cl_expr
            }
        }
        
        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr $($tail)*
        }
    };

    /*
    from ((arg: ty) expr) clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        from ($(($cl_arg:ident: $cl_ty:ty) $cl_expr:expr);*)
        $($tail:tt)*
    ) => {
        $(
            impl ::std::convert::From<$cl_ty> for $err_name {
                fn from($cl_arg: $cl_ty) -> $err_name {
                    $err_name::$var_name($cl_expr)
                }
            }
        )*

        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr $($tail)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! error_type_var_body {
    /*
    Base case: no more clauses.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        $disp:tt, $desc:tt, $cause:tt, $from:tt; {}
    ) => {
        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr,
            disp $disp, desc $desc, cause $cause, from $from
        }
    };

    /*
    disp (arg, fmt) expr;
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        $disp:tt, $desc:tt, $cause:tt, $from:tt; {
            disp ($cl_arg:ident, $cl_fmt:ident) $cl_body:expr;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr,
            (($cl_arg, $cl_fmt) $cl_body), $desc, $cause, $from;
            {$($tail)*}
        }
    };

    /*
    desc (arg) expr;
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        $disp:tt, $desc:tt, $cause:tt, $from:tt; {
            desc ($cl_arg:ident) $cl_body:expr;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr,
            $disp, (($cl_arg) $cl_body), $cause, $from;
            {$($tail)*}
        }
    };

    /*
    cause (arg) expr;
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        $disp:tt, $desc:tt, $cause:tt, $from:tt; {
            cause ($cl_arg:ident) $cl_body:expr;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr,
            $disp, $desc, (($cl_arg) $cl_body), $from;
            {$($tail)*}
        }
    };

    /*
    cause;
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        $disp:tt, $desc:tt, $cause:tt, $from:tt; {
            cause;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr,
            $disp, $desc, ((e) ::std::error::Error::cause(e)), $from;
            {$($tail)*}
        }
    };

    /*
    from (arg: Ty) expr; (first)
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        $disp:tt, $desc:tt, $cause:tt, (); {
            from ($cl_arg:ident: $cl_ty:ty) $cl_body:expr;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr,
            $disp, $desc, $cause, (($cl_arg: $cl_ty) $cl_body);
            {$($tail)*}
        }
    };

    /*
    from (arg: Ty) expr; (not first)
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $edi_tr:ident, $ede_tr:ident, $ec_tr:ident,
        $disp:tt, $desc:tt, $cause:tt, ($($from:tt)*); {
            from ($cl_arg:ident: $cl_ty:ty) $cl_body:expr;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $edi_tr, $ede_tr, $ec_tr,
            $disp, $desc, $cause, (($cl_arg: $cl_ty) $cl_body; $($from)*);
            {$($tail)*}
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! error_type_impl {
    (
        $(#[$($derive_tts:tt)*])*
        enum $err_name:ident {
            $($var_name:ident($var_ty:ty) $var_body:tt),+
            $(,)*
        }
    ) => {
        $(
            impl ::std::convert::From<$var_ty> for $err_name {
                fn from(value: $var_ty) -> $err_name {
                    $err_name::$var_name(value)
                }
            }
        )+
        
        impl ::std::fmt::Display for $err_name {
            fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
                match *self {
                    $(
                        $err_name::$var_name(ref v) => (self, v).error_fmt(fmt)
                    ),+
                }
            }
        }

        pub trait ErrorDisplay {
            fn error_fmt(&self, &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error>;
        }

        pub trait ErrorDescription<'a> {
            fn error_desc(&self) -> &'a str;
        }

        pub trait ErrorCause<'a> {
            fn error_cause(&self) -> ::std::option::Option<&'a ::std::error::Error>;
        }
        
        impl ::std::error::Error for $err_name {
            fn description(&self) -> &str {
                use self::ErrorDescription;
                match *self {
                    $(
                        $err_name::$var_name(ref v) => (self, v).error_desc()
                    ),+
                }
            }
            
            fn cause(&self) -> ::std::option::Option<&::std::error::Error> {
                use self::ErrorCause;
                match *self {
                    $(
                        $err_name::$var_name(ref v) => (self, v).error_cause()
                    ),+
                }
            }
        }
        
        $(
            error_type_var_body! {
                $err_name, $var_name, $var_ty,
                ErrorDisplay, ErrorDescription, ErrorCause,
                (), (), (), ();
                $var_body
            }
        )+
    };
}

/**
Constructs a reasonably well-featured error type from a concise description.

For details, see the crate documentation.
*/
#[macro_export]
macro_rules! error_type {
    (
        $(#[$($derive_tts:tt)*])*
        pub enum $err_name:ident {
            $($var_name:ident($var_ty:ty) $var_body:tt),+
            $(,)*
        }
    ) => {
        error_type_as_item! {
            $(#[$($derive_tts)*])*
            pub enum $err_name {
                $($var_name($var_ty)),+
            }
        }
        
        error_type_impl! {
            $(#[$($derive_tts)*])*
            enum $err_name {
                $($var_name($var_ty) $var_body),+
            }
        }
    };

    (
        $(#[$($derive_tts:tt)*])*
        enum $err_name:ident {
            $($var_name:ident($var_ty:ty) $var_body:tt),+
            $(,)*
        }
    ) => {
        error_type_as_item! {
            $(#[$($derive_tts)*])*
            enum $err_name {
                $($var_name($var_ty)),+
            }
        }
        
        error_type_impl! {
            $(#[$($derive_tts)*])*
            enum $err_name {
                $($var_name($var_ty) $var_body),+
            }
        }
    };
}
