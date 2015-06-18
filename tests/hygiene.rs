/*!
Ensures that the macro isn't tripped up by local re-definitions of various prelude types.
*/
#[macro_use] extern crate error_type;

type Debug = ();
type Display = ();
type Error = ();
type From = ();
type Option = ();
type Result = ();

error_type! {
    #[derive(Debug)]
    enum SomeError {
        Wat(Box<::std::error::Error + 'static>) {
            desc (e) e.description();
        }
    }
}
