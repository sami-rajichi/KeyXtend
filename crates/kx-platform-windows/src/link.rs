//! The browser link: the running Explorer opens an https link itself, so the browser starts with Explorer's rights and
//! never inherits our uiAccess (ADR-0041 decision 5). The rules are pure over `Os`; only `win` calls Windows.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows link runs this logic; tests cover it everywhere."
    )
)]

use kx_module_api::ports::{Link, LinkError};

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::WinWebLink;

/// The OS calls behind the link: the running shell on Windows, a recorder in tests.
pub(crate) trait Os {
    /// The running desktop shell's object.
    type Shell;

    /// The running desktop shell's object, or the code that refused it.
    fn shell(&self) -> Result<Self::Shell, i32>;

    /// Asks `shell` to open `link`, or gives the code that refused it.
    fn open(&self, shell: &Self::Shell, link: &str) -> Result<(), i32>;
}

/// Opens `link`, checked again by the port's rules so only an https link reaches the shell; `Unavailable` with no
/// shell, `Refused` when the shell says no.
pub(crate) fn open<O: Os>(os: &O, link: &str) -> Result<(), LinkError> {
    let link = Link::new(link.to_owned())?;
    let shell = os.shell().map_err(|_| LinkError::Unavailable)?;
    os.open(&shell, link.expose()).map_err(LinkError::Refused)
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use kx_module_api::ports::MAX_LINK_BYTES;

    use super::*;

    /// A code the fake shell refuses with.
    const CODE: i32 = -5;
    /// A link the port accepts.
    const GOOD: &str = "https://translate.example/?text=hello%20world";

    /// A recording shell: whether it is there and opens, how often it was asked for, and what it opened.
    #[derive(Default)]
    struct Fake {
        gone: bool,
        refuses: bool,
        asked: Cell<usize>,
        opened: RefCell<Vec<String>>,
    }

    impl Os for Fake {
        type Shell = ();

        fn shell(&self) -> Result<(), i32> {
            self.asked.set(self.asked.get() + 1);
            if self.gone { Err(CODE) } else { Ok(()) }
        }

        fn open(&self, (): &(), link: &str) -> Result<(), i32> {
            self.opened.borrow_mut().push(link.to_owned());
            if self.refuses { Err(CODE) } else { Ok(()) }
        }
    }

    #[test]
    fn an_https_link_reaches_the_shell_once_as_it_is() {
        let fake = Fake::default();
        assert_eq!(open(&fake, GOOD), Ok(()));
        assert_eq!(*fake.opened.borrow(), [GOOD]);
        assert_eq!(fake.asked.get(), 1);
    }

    #[test]
    fn a_link_the_port_refuses_never_asks_for_the_shell() {
        let long = format!("{GOOD}{}", "a".repeat(MAX_LINK_BYTES));
        let cases = [
            ("http://translate.example/", LinkError::NotHttps),
            ("file:///C:/Windows/System32/cmd.exe", LinkError::NotHttps),
            ("ms-settings:privacy", LinkError::NotHttps),
            ("HTTPS://translate.example/", LinkError::NotHttps),
            ("https://", LinkError::NotHttps),
            ("", LinkError::NotHttps),
            ("https://a.example/\" --flag", LinkError::BadChar),
            ("https://a.example/\\..\\cmd", LinkError::BadChar),
            ("https://a.example/\u{202E}", LinkError::BadChar),
            (long.as_str(), LinkError::TooLong),
        ];
        for (text, want) in cases {
            let fake = Fake::default();
            assert_eq!(open(&fake, text), Err(want), "{text:?}");
            assert_eq!(fake.asked.get(), 0, "{text:?}");
            assert!(fake.opened.borrow().is_empty(), "{text:?}");
        }
    }

    #[test]
    fn no_shell_is_unavailable_and_opens_nothing() {
        let fake = Fake {
            gone: true,
            ..Fake::default()
        };
        assert_eq!(open(&fake, GOOD), Err(LinkError::Unavailable));
        assert!(fake.opened.borrow().is_empty());
    }

    #[test]
    fn a_refused_open_is_refused_with_its_code() {
        let fake = Fake {
            refuses: true,
            ..Fake::default()
        };
        assert_eq!(open(&fake, GOOD), Err(LinkError::Refused(CODE)));
        assert_eq!(fake.opened.borrow().len(), 1, "asked once, never again");
    }

    #[test]
    fn no_error_names_the_link() {
        let gone = Fake {
            gone: true,
            ..Fake::default()
        };
        let refuses = Fake {
            refuses: true,
            ..Fake::default()
        };
        let secret = "https://translate.example/?text=secret";
        let got = [
            open(&gone, secret),
            open(&refuses, secret),
            open(&Fake::default(), "http://secret.example/"),
        ];
        for e in got.map(|g| g.expect_err("each case fails")) {
            let shown = format!("{e} {e:?}");
            assert!(!shown.contains("secret"), "{shown}");
        }
    }
}
