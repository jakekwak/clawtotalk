use dioxus::prelude::*;
use crate::models::{Message, MessageType};

#[derive(Props, Clone, PartialEq)]
pub struct ConversationHistoryProps {
    pub messages: Vec<Message>,
}

/// Conversation history component
/// Requirement 7.4: Display conversation history with auto-scroll
#[component]
pub fn ConversationHistory(props: ConversationHistoryProps) -> Element {
    rsx! {
        div {
            class: "conversation-history",
            style: "display: flex; flex-direction: column; gap: 12px; min-height: 100%;",
            
            if props.messages.is_empty() {
                div {
                    style: "text-align: center; color: #999; padding: 40px 20px;",
                    "No messages yet. Start a conversation by recording your voice!"
                }
            } else {
                for message in props.messages.iter() {
                    MessageBubble {
                        message: message.clone()
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct MessageBubbleProps {
    message: Message,
}

/// Individual message bubble component
/// Requirement 7.4: Distinguish user/assistant messages and show timestamps
#[component]
fn MessageBubble(props: MessageBubbleProps) -> Element {
    let (bg_color, text_color, align, icon) = match props.message.message_type {
        MessageType::User => ("#3b82f6", "white", "flex-end", "👤"),
        MessageType::Assistant => ("#10b981", "white", "flex-start", "🤖"),
        MessageType::System => ("#6b7280", "white", "center", "ℹ️"),
        MessageType::Error => ("#ef4444", "white", "center", "⚠️"),
    };
    
    let bubble_style = format!(
        "max-width: 70%; padding: 12px 16px; border-radius: 12px; \
         background: {}; color: {}; word-wrap: break-word; \
         box-shadow: 0 2px 4px rgba(0,0,0,0.1);",
        bg_color, text_color
    );
    
    let container_style = format!(
        "display: flex; flex-direction: column; align-items: {};",
        align
    );
    
    // Format timestamp
    let timestamp = props.message.timestamp.format("%H:%M:%S").to_string();
    
    rsx! {
        div {
            class: "message-container",
            style: "{container_style}",
            
            div {
                class: "message-bubble",
                style: "{bubble_style}",
                
                div {
                    style: "display: flex; align-items: center; gap: 8px; margin-bottom: 4px;",
                    span {
                        style: "font-size: 16px;",
                        "{icon}"
                    }
                    span {
                        style: "font-size: 12px; opacity: 0.8;",
                        "{timestamp}"
                    }
                }
                
                div {
                    style: "font-size: 14px; line-height: 1.5;",
                    "{props.message.content}"
                }
                
                // Show audio indicator if message has audio
                if props.message.audio_data.is_some() {
                    div {
                        style: "margin-top: 6px; font-size: 12px; opacity: 0.8;",
                        "🔊 Audio attached"
                    }
                }
            }
        }
    }
}
