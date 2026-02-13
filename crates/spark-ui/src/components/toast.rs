use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ToastLevel {
    Success,
    Error,
    Warning,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub level: ToastLevel,
}

#[derive(Clone, Copy)]
pub struct ToastContext {
    toasts: ReadSignal<Vec<Toast>>,
    set_toasts: WriteSignal<Vec<Toast>>,
    next_id: ReadSignal<u64>,
    set_next_id: WriteSignal<u64>,
}

impl ToastContext {
    pub fn push(&self, message: String, level: ToastLevel) {
        let currentId = self.next_id.get_untracked();
        self.set_next_id.set(currentId + 1);

        let toast = Toast {
            id: currentId,
            message,
            level,
        };

        self.set_toasts.update(|toasts| {
            toasts.push(toast);
        });

        let setToasts = self.set_toasts;
        let dismissId = currentId;
        set_timeout(
            move || {
                setToasts.update(|toasts| {
                    toasts.retain(|t| t.id != dismissId);
                });
            },
            std::time::Duration::from_secs(5),
        );
    }
}

/// Provides toast context and renders the toast container.
/// Place this once near the root of your app.
#[component]
pub fn ToastProvider(children: Children) -> impl IntoView {
    let (toasts, setToasts) = signal(Vec::<Toast>::new());
    let (nextId, setNextId) = signal(0u64);

    let ctx = ToastContext {
        toasts,
        set_toasts: setToasts,
        next_id: nextId,
        set_next_id: setNextId,
    };

    provide_context(ctx);

    view! {
        {children()}
        <div class="toast-container">
            <For
                each=move || toasts.get()
                key=|toast| toast.id
                let:toast
            >
                <div class=move || {
                    let levelClass = match toast.level {
                        ToastLevel::Success => "toast-success",
                        ToastLevel::Error => "toast-error",
                        ToastLevel::Warning => "toast-warning",
                    };
                    format!("toast {levelClass}")
                }>
                    {toast.message.clone()}
                </div>
            </For>
        </div>
    }
}
