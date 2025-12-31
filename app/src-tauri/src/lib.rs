#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().setup(|app| {
        if cfg!(debug_assertions) {
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            )?;
        }
        Ok(())
    });

    #[cfg(mobile)]
    {
        builder = builder
            .plugin(tauri_plugin_deep_link::init()) // This must be called before the sharetarget plugin
            .plugin(tauri_plugin_mobile_sharetarget::init())
            .setup(|_app| {
                #[cfg(target_os = "ios")]
                {
                    use tauri_plugin_deep_link::DeepLinkExt;
                    use tauri_plugin_mobile_sharetarget::{push_new_intent, IOS_DEEP_LINK_SCHEME};
                    let start_urls = _app.deep_link().get_current()?;
                    if let Some(urls) = start_urls {
                        println!("deep link URLs: {:?}", urls);
                        if let Some(url) = urls.first() {
                            if url.scheme().eq(IOS_DEEP_LINK_SCHEME.wait()) {
                                push_new_intent(url.to_string());
                            }
                        }
                    }

                    _app.deep_link().on_open_url(move |event| {
                        println!("got new url");
                        if let Some(url) = event.urls().first() {
                            if url.scheme().eq(IOS_DEEP_LINK_SCHEME.wait()) {
                                push_new_intent(url.to_string());
                            }
                        }
                    });
                }
                Ok(())
            });
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
