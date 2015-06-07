
# `error_type`

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

fn main() {}
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
