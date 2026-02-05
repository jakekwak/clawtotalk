use dioxus::prelude::*;
use crate::models::RecordingMode;

#[derive(Props, Clone, PartialEq)]
pub struct RecordingButtonProps {
    pub recording_mode: RecordingMode,
    pub is_recording: bool,
    pub on_toggle: EventHandler<()>,
    #[props(default = 0.0)]
    pub audio_level: f32,
}

/// Recording button component
/// Requirements 2.1, 7.2, 7.3: Supports touch and mouse input, provides visual feedback
#[component]
pub fn RecordingButton(props: RecordingButtonProps) -> Element {
    let mut is_pressed = use_signal(|| false);
    
    // Determine button state based on recording mode
    let button_text = match props.recording_mode {
        RecordingMode::Hold => {
            if props.is_recording {
                "🎤 Recording... (Release to stop)"
            } else {
                "🎤 Hold to Record"
            }
        }
        RecordingMode::Toggle => {
            if props.is_recording {
                "⏹️ Stop Recording"
            } else {
                "🎤 Start Recording"
            }
        }
        RecordingMode::Auto => {
            if props.is_recording {
                "🎤 Recording... (Auto)"
            } else {
                "🎤 Auto Mode (Speak to start)"
            }
        }
    };
    
    // Button styling based on recording state
    let button_color = if props.is_recording {
        "#ef4444" // Red when recording
    } else if *is_pressed.read() {
        "#3b82f6" // Blue when pressed
    } else {
        "#10b981" // Green when idle
    };
    
    let button_style = format!(
        "width: 200px; height: 200px; border-radius: 50%; border: none; \
         background: {}; color: white; font-size: 16px; font-weight: bold; \
         cursor: pointer; transition: all 0.2s; box-shadow: 0 4px 6px rgba(0,0,0,0.1); \
         display: flex; flex-direction: column; align-items: center; justify-content: center; \
         user-select: none; -webkit-user-select: none; -moz-user-select: none;",
        button_color
    );
    
    // Audio level indicator style
    let level_width = (props.audio_level * 100.0).min(100.0);
    let level_style = format!(
        "width: 80%; height: 8px; background: rgba(255,255,255,0.3); \
         border-radius: 4px; margin-top: 10px; overflow: hidden;"
    );
    let level_fill_style = format!(
        "width: {}%; height: 100%; background: rgba(255,255,255,0.8); \
         transition: width 0.1s;",
        level_width
    );
    
    rsx! {
        div {
            class: "recording-button-container",
            style: "display: flex; flex-direction: column; align-items: center;",
            
            // Main recording button
            button {
                class: "recording-button",
                style: "{button_style}",
                
                // Mouse events
                onclick: move |_| {
                    // Requirement 7.2: Handle mouse input
                    match props.recording_mode {
                        RecordingMode::Toggle => {
                            props.on_toggle.call(());
                        }
                        RecordingMode::Auto => {
                            // Auto mode doesn't use button clicks
                        }
                        RecordingMode::Hold => {
                            // Hold mode uses mousedown/mouseup
                        }
                    }
                },
                
                onmousedown: move |_| {
                    // Requirement 7.2: Handle mouse input for Hold mode
                    is_pressed.set(true);
                    if props.recording_mode == RecordingMode::Hold && !props.is_recording {
                        props.on_toggle.call(());
                    }
                },
                
                onmouseup: move |_| {
                    is_pressed.set(false);
                    if props.recording_mode == RecordingMode::Hold && props.is_recording {
                        props.on_toggle.call(());
                    }
                },
                
                onmouseleave: move |_| {
                    is_pressed.set(false);
                    if props.recording_mode == RecordingMode::Hold && props.is_recording {
                        props.on_toggle.call(());
                    }
                },
                
                // Touch events for mobile
                ontouchstart: move |_| {
                    // Requirement 7.2: Handle touch input
                    is_pressed.set(true);
                    if props.recording_mode == RecordingMode::Hold && !props.is_recording {
                        props.on_toggle.call(());
                    }
                },
                
                ontouchend: move |_| {
                    is_pressed.set(false);
                    if props.recording_mode == RecordingMode::Hold && props.is_recording {
                        props.on_toggle.call(());
                    }
                },
                
                div {
                    style: "text-align: center;",
                    "{button_text}"
                }
                
                // Audio level indicator
                // Requirement 7.3: Visual feedback during recording
                if props.is_recording {
                    div {
                        class: "audio-level",
                        style: "{level_style}",
                        div {
                            class: "audio-level-fill",
                            style: "{level_fill_style}"
                        }
                    }
                }
            }
            
            // Mode indicator
            div {
                style: "margin-top: 15px; font-size: 14px; color: #666;",
                "Mode: {props.recording_mode:?}"
            }
        }
    }
}
