use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ErrorNotificationProps {
    pub message: String,
    #[props(default)]
    pub on_dismiss: Option<EventHandler<()>>,
    #[props(default)]
    pub on_retry: Option<EventHandler<()>>,
    #[props(default = 5000)]
    pub auto_dismiss_ms: u64,
}

/// Error notification component
/// Requirements 10.1, 10.2, 10.3: Display error messages with recovery actions
#[component]
pub fn ErrorNotification(props: ErrorNotificationProps) -> Element {
    let mut is_visible = use_signal(|| true);
    
    // Auto-dismiss after specified time
    // Requirement: Auto-dismiss functionality
    use_effect(move || {
        if props.auto_dismiss_ms > 0 {
            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(props.auto_dismiss_ms)).await;
                is_visible.set(false);
                if let Some(on_dismiss) = props.on_dismiss {
                    on_dismiss.call(());
                }
            });
        }
    });
    
    if !*is_visible.read() {
        return rsx! { div {} };
    }
    
    let notification_style = "display: flex; align-items: center; justify-content: space-between; \
                              padding: 12px 16px; background: #fee2e2; border: 1px solid #ef4444; \
                              border-radius: 6px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); \
                              animation: slideIn 0.3s ease-out;";
    
    rsx! {
        div {
            class: "error-notification",
            style: "{notification_style}",
            
            div {
                style: "display: flex; align-items: center; gap: 10px; flex: 1;",
                
                span {
                    style: "font-size: 20px;",
                    "⚠️"
                }
                
                span {
                    style: "color: #991b1b; font-size: 14px;",
                    "{props.message}"
                }
            }
            
            div {
                style: "display: flex; align-items: center; gap: 8px;",
                
                // Retry button if handler provided
                // Requirement 10.1, 10.2, 10.3: Recovery action buttons
                if let Some(on_retry) = props.on_retry {
                    button {
                        onclick: move |_| {
                            on_retry.call(());
                            is_visible.set(false);
                        },
                        style: "padding: 6px 12px; background: #ef4444; color: white; border: none; \
                                border-radius: 4px; font-size: 12px; cursor: pointer; font-weight: 500;",
                        "Retry"
                    }
                }
                
                // Dismiss button
                if let Some(on_dismiss) = props.on_dismiss {
                    button {
                        onclick: move |_| {
                            is_visible.set(false);
                            on_dismiss.call(());
                        },
                        style: "padding: 6px 12px; background: transparent; color: #991b1b; \
                                border: 1px solid #991b1b; border-radius: 4px; font-size: 12px; \
                                cursor: pointer; font-weight: 500;",
                        "✕"
                    }
                }
            }
        }
    }
}
