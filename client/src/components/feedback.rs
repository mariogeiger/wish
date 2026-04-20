use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: u32,
    pub title: String,
    pub message: String,
    pub kind: ToastKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToastKind {
    Success,
    Error,
    Info,
}

impl ToastKind {
    fn css_class(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Info => "info",
        }
    }
}

#[component]
pub fn ToastContainer(
    toasts: ReadSignal<Vec<Toast>>,
    set_toasts: WriteSignal<Vec<Toast>>,
) -> impl IntoView {
    view! {
        <div class="toast-container">
            {move || {
                toasts
                    .get()
                    .into_iter()
                    .map(|t| {
                        let class = format!("toast {}", t.kind.css_class());
                        let id = t.id;
                        let on_close = move |_| {
                            set_toasts.update(|ts| ts.retain(|x| x.id != id));
                        };
                        view! {
                            <div class=class>
                                <button class="toast-close" on:click=on_close aria-label="close">"×"</button>
                                <strong>{t.title}</strong>
                                <div inner_html=t.message />
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}

static NEXT_TOAST_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn add_toast(
    set_toasts: &WriteSignal<Vec<Toast>>,
    title: &str,
    message: &str,
    kind: ToastKind,
) {
    let id = NEXT_TOAST_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let toast = Toast {
        id,
        title: title.to_string(),
        message: message.to_string(),
        kind,
    };
    let auto_dismiss = kind != ToastKind::Error;
    set_toasts.update(|ts| ts.push(toast));

    if auto_dismiss {
        let set_toasts = *set_toasts;
        wasm_bindgen_futures::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(4000).await;
            set_toasts.update(|ts| ts.retain(|t| t.id != id));
        });
    }
}
