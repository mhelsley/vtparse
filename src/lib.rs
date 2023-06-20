/// vtparser library for Rust
/// Wrapper made using bindgen
/// Packaged by Matt Helsley <matt.helsley+oss@gmail.com>
///
/// Wraps the public domain C parser originally written by:
///     Joshua Haberman
/// For the original source code:
///     https://github.com/haberman/vtparse.git
/// For the state machine documentation see:
///     https://vt100.net/emu/dec_ansi_parser
///
use std::ffi::c_void;

#[allow(non_camel_case_types)]
#[allow(unused)]
mod vtparse_c {
    include!(concat!(env!("OUT_DIR"), "/vtparse_bindings.rs"));
}

pub type Action = vtparse_c::vtparse_action_t;
type CParser = vtparse_c::vtparse_t;

impl CParser {
    fn parse(&mut self, data: *const str, len: usize) {
        unsafe {
            vtparse_c::vtparse(self, data as *const u8, len as i32);
        }
    }
}

pub struct Parser {
    inner: CParser,
}

use container_of::container_of;

impl Parser {
    // Wrap the Rust-friendly callbacks
    extern "C" fn wrapper(cparser: *mut CParser, action: vtparse_c::vtparse_action_t, c: u8) {
        if let Err(err) = std::panic::catch_unwind(|| {
            let parser = unsafe { &mut *container_of!(cparser, Parser, inner) };
            let cb_ptr = (*cparser).user_data as *mut c_void;
            let cb = cb_ptr as *mut dyn FnMut(&mut Parser, Action, u8);
            (*cb)(parser, action, c);
        }) {
            // Code here must be panic-free.
            #[cfg(not(feature = "unwind"))]
            {
                // Sane things to do:
                // log failure and/or kill the program
                eprintln!("{:?}", err);
                // Abort is safe because it doesn't unwind.
                std::process::abort();
            }
            #[cfg(feature = "unwind")]
            {

                // We can clobber parser.user_data because we're panic-ing
                let err_ptr = std::ptr::addr_of!(err) as *mut c_void;
                (*parser).user_data = err_ptr as *mut c_void;
            }
        }
    }

    pub fn new(cb: &mut dyn FnMut(&mut Parser, Action, u8)) -> Self {
        let mut cparser = std::mem::MaybeUninit::<CParser>::zeroed();
        unsafe {
            vtparse_c::vtparse_init(cparser.as_mut_ptr(), Some(Self::wrapper));
        }
        let cparser = unsafe { cparser.assume_init() };
        let mut s = Self {
            inner: cparser,
        };
        s.inner.user_data = cb as *mut dyn FnMut(&mut Parser, Action, u8) as *mut c_void;
        s
    }
    pub fn parse(&mut self, data: *const str, len: usize) {
        self.inner.parse(data, len);
        #[cfg(feature = "unwind")]
        {
            let err_ptr = self.inner.user_data;
            let err: &mut std::panic::PanicInfo = &mut *err_ptr;
            panic::resume_unwind(err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_only() {
        fn cb(_parser: &mut Parser, action: Action, _c: u8) {
            match action {
                Action::Print => {}
                _ => assert!(false, "Unexpected action {:?}", action),
            }
        }
        let mut parser = Parser::new(cb);

        let data = "plain text";
        parser.parse(data, data.len());
    }

    #[test]
    fn println_only() {
        fn cb(_parser: &mut Parser, action: Action, _c: u8) {
            match action {
                Action::Print | Action::Execute => {}
                _ => assert!(false, "Unexpected action {:?}", action),
            }
        }
        let mut parser = Parser::new(cb);

        let data = "plain\ntext\n";
        parser.parse(data, data.len());
    }

    #[test]
    fn csi_sgr_reset() {
        fn cb(_parser: &mut Parser, action: Action, _c: u8) {
            match action {
                Action::CSIDispatch => {}
                _ => assert!(false, "Unexpected action {:?}", action),
            }
        }
        let mut parser = Parser::new(cb);

        let data = "\x1B[0m";
        parser.parse(data, data.len());
    }

    #[test]
    fn osc_seq() {
        fn cb(_parser: &mut Parser, action: Action, _c: u8) {
            match action {
                Action::OSCStart
                | Action::OSCPut
                | Action::OSCEnd
                | Action::EscDispatch
                | Action::Print => {}
                _ => assert!(false, "Unexpected action {:?}", action),
            }
        }
        let mut parser = Parser::new(cb);

        let data = "\x1B]8;key=foo;https://example.co\x1B\\\\link\x1B]8;;\x1B\\\\";
        parser.parse(data, data.len());
    }

    #[test]
    fn empty() {
        fn cb(_parser: &mut Parser, _action: Action, _c: u8) {}
        let mut parser = Parser::new(cb);

        let data = "";
        parser.parse(data, data.len());
    }

    #[test]
    fn csi_ignore() {
        fn cb(_parser: &mut Parser, action: Action, _c: u8) {
            match action {
                Action::CSIDispatch => {}
                _ => assert!(false, "Unexpected action {:?}", action),
            }
        }
        let mut parser = Parser::new(cb);

        let data = "\x1B[\x3A";
        parser.parse(data, data.len());
    }

    #[cfg(not(feature = "unwind"))]
    #[test]
    #[ignore]
    fn broken_callback() {
        fn cb(_parser: &mut Parser, _action: Action, _c: u8) {
            panic!("Breaks");
        }
        let mut parser = Parser::new(cb);

        let data = "!";
        parser.parse(data, data.len());
    }

    #[cfg(feature = "unwind")]
    #[test]
    #[should_panic(expected = "Breaks")]
    fn broken_callback() {
        fn cb(_parser: &mut Parser, _action: Action, _c: u8) {
            panic!("Breaks");
        }
        let mut parser = Parser::new(cb);

        let data = "!";
        parser.parse(data, data.len());
    }
}
