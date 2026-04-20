use leptos::prelude::*;
use wish_shared::{SetWishRequest, WishData};

use crate::NavBar;
use crate::api;
use crate::components::feedback::{ToastContainer, ToastKind, add_toast};
use crate::components::slider::WishSliders;
use crate::i18n::{translations, use_lang};

#[component]
pub fn WishPage(key: String) -> impl IntoView {
    let lang = use_lang();
    let (toasts, set_toasts) = signal(Vec::new());
    let (data, set_data) = signal(None::<WishData>);
    let (wish, set_wish) = signal(Vec::<i32>::new());
    let (saving, set_saving) = signal(false);
    let key = StoredValue::new(key);

    // Fetch wish data
    {
        let k = key.get_value();
        wasm_bindgen_futures::spawn_local(async move {
            match api::get::<WishData>(&format!("/api/wish/{k}")).await {
                Ok(d) => {
                    set_wish.set(d.wish.clone());
                    set_data.set(Some(d));

                    // Delay marking visited so link-scanners that execute JS
                    // but close the page quickly don't trigger a false visit.
                    gloo_timers::future::TimeoutFuture::new(5000).await;
                    let _ = api::post::<_, serde_json::Value>(
                        &format!("/api/wish/{k}/visit"),
                        &serde_json::json!({}),
                    )
                    .await;
                }
                Err(e) => {
                    let t = translations(lang.get());
                    add_toast(
                        &set_toasts,
                        t.error,
                        &format!("{}{e}", t.failed_to_load),
                        ToastKind::Error,
                    );
                }
            }
        });
    }

    let on_save = move |_: web_sys::MouseEvent| {
        set_saving.set(true);
        let k = key.get_value();
        let w = wish.get();
        wasm_bindgen_futures::spawn_local(async move {
            let req = SetWishRequest { wish: w };
            let t = translations(lang.get());
            match api::put::<_, serde_json::Value>(&format!("/api/wish/{k}"), &req).await {
                Ok(_) => {
                    add_toast(&set_toasts, t.saved, t.wish_saved_body, ToastKind::Success);
                }
                Err(e) => {
                    add_toast(&set_toasts, t.error, &e, ToastKind::Error);
                }
            }
            set_saving.set(false);
        });
    };

    view! {
        <ToastContainer toasts=toasts />
        <div class="container">
            <h1>"Wish"</h1>
            <NavBar />

            {move || {
                let t = translations(lang.get());
                if let Some(d) = data.get() {
                    view! {
                        <p><strong>{t.wish_activity}</strong>{d.name.clone()}</p>
                        <p class="muted">{d.mail.clone()}</p>

                        <WishSliders
                            slot_names=d.slots.iter().map(|s| s.name.clone()).collect()
                            wish=wish
                            set_wish=set_wish
                        />

                        <div class="btn-row">
                            <button on:click=on_save disabled=move || saving.get()>
                                {move || {
                                    let t = translations(lang.get());
                                    if saving.get() { t.wish_saving } else { t.wish_save }
                                }}
                            </button>
                        </div>
                    }
                    .into_any()
                } else {
                    view! { <p>{t.wish_loading}</p> }.into_any()
                }
            }}
        </div>
    }
}
