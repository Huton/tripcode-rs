#[cfg(feature = "i18n")]
#[macro_use]
mod gettext;
#[cfg(not(feature = "i18n"))]
#[macro_use]
mod gettext {
    #[macro_export]
    macro_rules! tr {
        ($($arg:expr),*) => ($($arg),*);
    }
    #[macro_export]
    macro_rules! i18n_init {
        () => {}
    }
}

pub use self::gettext::*;
#[cfg(feature = "i18n")]
pub use self::gettext::tr::tr;
