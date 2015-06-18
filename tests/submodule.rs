/*!
Check to make sure the macro works in non-root modules.
*/
#[macro_use] extern crate error_type;

mod inner {
    error_type! {
        #[derive(Debug)]
        enum Error {
            Other(String) {
                desc(e) &**e;
            },
        }
    }
}
