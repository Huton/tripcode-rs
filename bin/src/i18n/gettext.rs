pub extern crate tr;
pub extern crate i18n_embed;

use rust_embed::{RustEmbed};

#[derive(Debug, RustEmbed)]
#[folder = "i18n/mo"]
pub struct Localizations;

#[macro_export]
macro_rules! i18n_init {
    () => {
        let requested_languages = $crate::i18n_embed::DesktopLanguageRequester::requested_languages();
        let language_loader = $crate::i18n_embed::gettext::gettext_language_loader!();
        let _result = $crate::i18n_embed::select(
            &language_loader, &$crate::i18n::Localizations, &requested_languages);
    }
}
