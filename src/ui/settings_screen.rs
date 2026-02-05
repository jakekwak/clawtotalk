use dioxus::prelude::*;
use crate::models::{Settings, ConnectionType, RecordingMode};

#[derive(Props, Clone, PartialEq)]
pub struct SettingsScreenProps {
    pub settings: Settings,
    pub on_save: EventHandler<Settings>,
}

/// Settings screen component
/// Requirements 8.1, 8.2, 8.3, 8.5: Configure server URL, connection type, recording mode, VAD sensitivity
#[component]
pub fn SettingsScreen(props: SettingsScreenProps) -> Element {
    // Local state for form inputs
    let mut server_url = use_signal(|| props.settings.server_config.server_url.clone());
    let mut connection_type = use_signal(|| props.settings.server_config.connection_type.clone());
    let mut auth_token = use_signal(|| props.settings.server_config.auth_token.clone().unwrap_or_default());
    let mut timeout_seconds = use_signal(|| props.settings.server_config.timeout_seconds);
    let mut recording_mode = use_signal(|| props.settings.recording_mode.clone());
    let mut vad_threshold = use_signal(|| props.settings.vad_settings.threshold);
    let mut vad_silence_duration = use_signal(|| props.settings.vad_settings.silence_duration_ms);
    
    let form_style = "display: flex; flex-direction: column; gap: 20px; max-width: 600px;";
    let section_style = "padding: 20px; border: 1px solid #e0e0e0; border-radius: 8px; background: white;";
    let label_style = "display: block; font-weight: bold; margin-bottom: 8px; color: #333;";
    let input_style = "width: 100%; padding: 10px; border: 1px solid #ccc; border-radius: 4px; font-size: 14px; box-sizing: border-box;";
    let button_style = "padding: 12px 24px; background: #3b82f6; color: white; border: none; border-radius: 4px; font-size: 16px; font-weight: bold; cursor: pointer;";
    
    rsx! {
        div {
            class: "settings-form",
            style: "{form_style}",
            
            // Server Configuration Section
            // Requirement 8.1: Server URL configuration
            div {
                class: "settings-section",
                style: "{section_style}",
                
                h2 {
                    style: "margin-top: 0; margin-bottom: 15px; font-size: 18px;",
                    "Server Configuration"
                }
                
                div {
                    style: "margin-bottom: 15px;",
                    label {
                        style: "{label_style}",
                        "Server URL"
                    }
                    input {
                        r#type: "text",
                        style: "{input_style}",
                        value: "{server_url}",
                        placeholder: "http://localhost:8080 or https://your-domain.com",
                        oninput: move |evt| {
                            server_url.set(evt.value().clone());
                        }
                    }
                    div {
                        style: "font-size: 12px; color: #666; margin-top: 4px;",
                        "Enter your OpenClaw server URL (Tailscale IP or public URL)"
                    }
                }
                
                // Requirement 8.1: Connection type selection
                div {
                    style: "margin-bottom: 15px;",
                    label {
                        style: "{label_style}",
                        "Connection Type"
                    }
                    
                    div {
                        style: "display: flex; flex-direction: column; gap: 8px;",
                        
                        label {
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                            input {
                                r#type: "radio",
                                name: "connection_type",
                                checked: *connection_type.read() == ConnectionType::Tailscale,
                                onchange: move |_| {
                                    connection_type.set(ConnectionType::Tailscale);
                                }
                            }
                            span { "Tailscale VPN" }
                        }
                        
                        label {
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                            input {
                                r#type: "radio",
                                name: "connection_type",
                                checked: *connection_type.read() == ConnectionType::PublicUrl,
                                onchange: move |_| {
                                    connection_type.set(ConnectionType::PublicUrl);
                                }
                            }
                            span { "Public URL (Cloudflare Tunnel)" }
                        }
                        
                        label {
                            style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                            input {
                                r#type: "radio",
                                name: "connection_type",
                                checked: *connection_type.read() == ConnectionType::LocalNetwork,
                                onchange: move |_| {
                                    connection_type.set(ConnectionType::LocalNetwork);
                                }
                            }
                            span { "Local Network" }
                        }
                    }
                }
                
                div {
                    style: "margin-bottom: 15px;",
                    label {
                        style: "{label_style}",
                        "Auth Token (Optional)"
                    }
                    input {
                        r#type: "password",
                        style: "{input_style}",
                        value: "{auth_token}",
                        placeholder: "Enter authentication token if required",
                        oninput: move |evt| {
                            auth_token.set(evt.value().clone());
                        }
                    }
                }
                
                div {
                    label {
                        style: "{label_style}",
                        "Timeout (seconds)"
                    }
                    input {
                        r#type: "number",
                        style: "{input_style}",
                        value: "{timeout_seconds}",
                        min: "5",
                        max: "120",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<u64>() {
                                timeout_seconds.set(val);
                            }
                        }
                    }
                }
            }
            
            // Recording Mode Section
            // Requirement 8.2: Recording mode selection
            div {
                class: "settings-section",
                style: "{section_style}",
                
                h2 {
                    style: "margin-top: 0; margin-bottom: 15px; font-size: 18px;",
                    "Recording Mode"
                }
                
                div {
                    style: "display: flex; flex-direction: column; gap: 8px;",
                    
                    label {
                        style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                        input {
                            r#type: "radio",
                            name: "recording_mode",
                            checked: *recording_mode.read() == RecordingMode::Hold,
                            onchange: move |_| {
                                recording_mode.set(RecordingMode::Hold);
                            }
                        }
                        span { "Hold - Press and hold to record" }
                    }
                    
                    label {
                        style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                        input {
                            r#type: "radio",
                            name: "recording_mode",
                            checked: *recording_mode.read() == RecordingMode::Toggle,
                            onchange: move |_| {
                                recording_mode.set(RecordingMode::Toggle);
                            }
                        }
                        span { "Toggle - Click to start, click again to stop" }
                    }
                    
                    label {
                        style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                        input {
                            r#type: "radio",
                            name: "recording_mode",
                            checked: *recording_mode.read() == RecordingMode::Auto,
                            onchange: move |_| {
                                recording_mode.set(RecordingMode::Auto);
                            }
                        }
                        span { "Auto - Automatic voice detection" }
                    }
                }
            }
            
            // VAD Settings Section
            // Requirement 8.3: VAD sensitivity adjustment
            div {
                class: "settings-section",
                style: "{section_style}",
                
                h2 {
                    style: "margin-top: 0; margin-bottom: 15px; font-size: 18px;",
                    "Voice Activity Detection"
                }
                
                div {
                    style: "margin-bottom: 15px;",
                    label {
                        style: "{label_style}",
                        "Sensitivity Threshold: {vad_threshold:.3}"
                    }
                    input {
                        r#type: "range",
                        style: "width: 100%;",
                        min: "0.001",
                        max: "0.1",
                        step: "0.001",
                        value: "{vad_threshold}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<f32>() {
                                vad_threshold.set(val);
                            }
                        }
                    }
                    div {
                        style: "font-size: 12px; color: #666; margin-top: 4px;",
                        "Lower values = more sensitive (detects quieter sounds)"
                    }
                }
                
                div {
                    label {
                        style: "{label_style}",
                        "Silence Duration (ms): {vad_silence_duration}"
                    }
                    input {
                        r#type: "range",
                        style: "width: 100%;",
                        min: "500",
                        max: "3000",
                        step: "100",
                        value: "{vad_silence_duration}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<u64>() {
                                vad_silence_duration.set(val);
                            }
                        }
                    }
                    div {
                        style: "font-size: 12px; color: #666; margin-top: 4px;",
                        "How long to wait after silence before stopping recording"
                    }
                }
            }
            
            // Save Button
            // Requirement 8.5: Save settings
            div {
                style: "display: flex; justify-content: center;",
                button {
                    style: "{button_style}",
                    onclick: move |_| {
                        let new_settings = Settings {
                            server_config: crate::models::ServerConfig {
                                server_url: server_url.read().clone(),
                                connection_type: connection_type.read().clone(),
                                auth_token: if auth_token.read().is_empty() {
                                    None
                                } else {
                                    Some(auth_token.read().clone())
                                },
                                timeout_seconds: *timeout_seconds.read(),
                            },
                            recording_mode: recording_mode.read().clone(),
                            vad_settings: crate::models::VadSettings {
                                threshold: *vad_threshold.read(),
                                window_size: props.settings.vad_settings.window_size,
                                silence_duration_ms: *vad_silence_duration.read(),
                            },
                            audio_settings: props.settings.audio_settings.clone(),
                        };
                        props.on_save.call(new_settings);
                    },
                    "💾 Save Settings"
                }
            }
        }
    }
}
