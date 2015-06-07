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
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident
    ) => {
        // Done.
    };

    /*
    desc () clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        desc ()
        $($tail:tt)*
    ) => {
        impl<'a> $ed_tr<'a> for (&'a $err_name, &'a $var_ty) {
            fn error_desc(&self) -> &'a str {
                std::error::Error::description(self.1)
            }
        }

        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr $($tail)*
        }
    };

    /*
    desc ((arg) expr) clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        desc (($desc_arg:ident) $desc_expr:expr)
        $($tail:tt)*
    ) => {
        impl<'a> $ed_tr<'a> for (&'a $err_name, &'a $var_ty) {
            fn error_desc(&self) -> &'a str {
                let $desc_arg = self.1;
                $desc_expr
            }
        }
        
        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr $($tail)*
        }
    };

    /*
    cause () clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        cause ()
        $($tail:tt)*
    ) => {
        impl<'a> $ec_tr<'a> for (&'a $err_name, &'a $var_ty) {
            fn error_cause(&self) -> Option<&'a std::error::Error> {
                None
            }
        }

        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr $($tail)*
        }
    };

    /*
    cause ((arg) expr) clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        cause (($cl_arg:ident) $cl_expr:expr)
        $($tail:tt)*
    ) => {
        impl<'a> $ec_tr<'a> for (&'a $err_name, &'a $var_ty) {
            fn error_cause(&self) -> Option<&'a std::error::Error> {
                let $cl_arg = self.1;
                $cl_expr
            }
        }
        
        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr $($tail)*
        }
    };

    /*
    from ((arg: ty) expr) clause.
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        from ($(($cl_arg:ident: $cl_ty:ty) $cl_expr:expr);*)
        $($tail:tt)*
    ) => {
        $(
            impl From<$cl_ty> for $err_name {
                fn from($cl_arg: $cl_ty) -> $err_name {
                    $err_name::$var_name($cl_expr)
                }
            }
        )*

        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr $($tail)*
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
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        $desc:tt, $cause:tt, $from:tt; {}
    ) => {
        error_type_var_body_emit! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr,
            desc $desc, cause $cause, from $from
        }
    };

    /*
    desc (arg) expr;
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        $desc:tt, $cause:tt, $from:tt; {
            desc ($cl_arg:ident) $cl_body:expr;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr,
            (($cl_arg) $cl_body), $cause, $from;
            {$($tail)*}
        }
    };

    /*
    cause (arg) expr;
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        $desc:tt, $cause:tt, $from:tt; {
            cause ($cl_arg:ident) $cl_body:expr;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr,
            $desc, (($cl_arg) $cl_body), $from;
            {$($tail)*}
        }
    };

    /*
    cause;
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        $desc:tt, $cause:tt, $from:tt; {
            cause;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr,
            $desc, ((e) std::error::Error::cause(e)), $from;
            {$($tail)*}
        }
    };

    /*
    from (arg: Ty) expr; (first)
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        $desc:tt, $cause:tt, (); {
            from ($cl_arg:ident: $cl_ty:ty) $cl_body:expr;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr,
            $desc, $cause, (($cl_arg: $cl_ty) $cl_body);
            {$($tail)*}
        }
    };

    /*
    from (arg: Ty) expr; (not first)
    */
    (
        $err_name:ident, $var_name:ident, $var_ty:ty, $ed_tr:ident, $ec_tr:ident,
        $desc:tt, $cause:tt, ($($from:tt)*); {
            from ($cl_arg:ident: $cl_ty:ty) $cl_body:expr;
            $($tail:tt)*
        }
    ) => {
        error_type_var_body! {
            $err_name, $var_name, $var_ty, $ed_tr, $ec_tr,
            $desc, $cause, (($cl_arg: $cl_ty) $cl_body; $($from)*);
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
            impl From<$var_ty> for $err_name {
                fn from(value: $var_ty) -> $err_name {
                    $err_name::$var_name(value)
                }
            }
        )+
        
        impl std::fmt::Display for $err_name {
            fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
                match *self {
                    $(
                        $err_name::$var_name(ref v) => std::fmt::Display::fmt(v, fmt)
                    ),+
                }
            }
        }

        pub trait ErrorDescription<'a> {
            fn error_desc(&self) -> &'a str;
        }

        pub trait ErrorCause<'a> {
            fn error_cause(&self) -> Option<&'a std::error::Error>;
        }
        
        impl std::error::Error for $err_name {
            fn description(&self) -> &str {
                use ErrorDescription;
                match *self {
                    $(
                        $err_name::$var_name(ref v) => (self, v).error_desc()
                    ),+
                }
            }
            
            fn cause(&self) -> Option<&std::error::Error> {
                use ErrorCause;
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
                ErrorDescription, ErrorCause,
                (), (), ();
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

    (
        $(#[$($derive_tts:tt)*])*
        pub enum $err_name:ident {
            $($body_tts:tt)*
        }
        
        $($tail:tt)*
    ) => {
        error_type_impl! {
            $(#[$($derive_tts)*])*
            (pub) enum $err_name {
                $($body_tts)*
            }
        }
        
        error_type! { $($tail)* }
    };
}
