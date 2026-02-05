use dioxus::prelude::*;
use crate::models::ConnectionStatus;

#[derive(Props, Clone, PartialEq)]
pub struct ConnectionStatusDisplayProps {
    pub status: ConnectionStatus,
    #[props(default = None)]
    pub response_time_ms: Option<u64>,
    #[props(default)]
    pub on_reconnect: Option<EventHandler<()>>,
}

/// Connection status display component
/// Requirements 8.6, 9.4: Show connection status and provide reconnect option
#[component]
pub fn ConnectionStatusDisplay(props: ConnectionStatusDisplayProps) -> Element {
    let (icon, color, text) = match &props.status {
        ConnectionStatus::Connected => ("✅", "#10b981", "Connected"),
        ConnectionStatus::Connecting => ("🔄", "#f59e0b", "Connecting..."),
        ConnectionStatus::Disconnected => ("⭕", "#6b7280", "Disconnected"),
        ConnectionStatus::Error(msg) => ("❌", "#ef4444", msg.as_str()),
    };
    
    let container_style = format!(
        "display: flex; align-items: center; gap: 8px; padding: 8px 12px; \
         border-radius: 6px; background: {}20; border: 1px solid {};",
        color, color
    );
    
    rsx! {
        div {
            class: "connection-status",
            style: "{container_style}",
            
            span {
                style: "font-size: 16px;",
                "{icon}"
            }
            
            div {
                style: "display: flex; flex-direction: column; gap: 2px;",
                
                span {
                    style: "font-size: 14px; font-weight: 500; color: {color};",
                    "{text}"
                }
                
                // Show response time if connected
                if let Some(response_time) = props.response_time_ms {
                    if props.status == ConnectionStatus::Connected {
                        span {
                            style: "font-size: 11px; color: #666;",
                            "{response_time}ms"
                        }
                    }
                }
            }
            
            // Reconnect button for disconnected/error states
            // Requirement 9.4: Provide reconnect option
            if matches!(props.status, ConnectionStatus::Disconnected | ConnectionStatus::Error(_)) {
                if let Some(on_reconnect) = props.on_reconnect {
                    button {
                        onclick: move |_| {
                            on_reconnect.call(());
                        },
                        style: "padding: 4px 8px; font-size: 12px; background: {color}; color: white; border: none; border-radius: 4px; cursor: pointer; margin-left: 8px;",
                        "Reconnect"
                    }
                }
            }
        }
    }
}
