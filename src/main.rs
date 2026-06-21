pub mod core;
pub mod gui;
pub mod net;
pub mod utils;
use qmetaobject::prelude::*;
use qmetaobject::QUrl;

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en");

pub fn translate(key: &str) -> String {
    rust_i18n::t!(key, locale = "en").into_owned()
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();
    println!("Starting Assetto Corsa Cloud Server Manager GUI...");

    let mut engine = QmlEngine::new();
    engine.load_url(QUrl::from(QString::from(
        "qrc:/qt/qml/com/acmanager/qml/main.qml",
    )));
    engine.exec();
}
