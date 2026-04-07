use leptos::prelude::*;
use wish_shared::{SetWishRequest, WishData};

use crate::api;
use crate::components::feedback::{ToastContainer, ToastKind, add_toast};
use crate::components::slider::WishSliders;

#[component]
pub fn WishPage(key: String) -> impl IntoView {
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
                }
                Err(e) => {
                    add_toast(
                        &set_toasts,
                        "Error",
                        &format!("Failed to load: {e}"),
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
            match api::put::<_, serde_json::Value>(&format!("/api/wish/{k}"), &req).await {
                Ok(_) => {
                    add_toast(
                        &set_toasts,
                        "Saved",
                        "Your wish has been saved.",
                        ToastKind::Success,
                    );
                }
                Err(e) => {
                    add_toast(&set_toasts, "Error", &e, ToastKind::Error);
                }
            }
            set_saving.set(false);
        });
    };

    view! {
        <ToastContainer toasts=toasts />
        <div class="container">
            <h1>"Wish"</h1>
            <nav>
                <a href="/">"Home"</a>
                <a href="/help">"Help"</a>
            </nav>

            {move || {
                if let Some(d) = data.get() {
                    view! {
                        <p><strong>"Activity: "</strong>{d.name.clone()}</p>
                        <p class="muted">{d.mail.clone()}</p>

                        <WishSliders
                            slot_names=d.slots.iter().map(|s| s.name.clone()).collect()
                            wish=wish
                            set_wish=set_wish
                        />

                        <div class="btn-row">
                            <button on:click=on_save disabled=move || saving.get()>
                                {move || if saving.get() { "Saving..." } else { "Save" }}
                            </button>
                        </div>
                    }
                    .into_any()
                } else {
                    view! { <p>"Loading..."</p> }.into_any()
                }
            }}
        </div>
    }
}
