use crate::error::AudioError;
use log::{error, info, warn};

#[cfg(target_os = "android")]
use jni::{
    objects::{JClass, JObject, JString, JValue},
    sys::{jboolean, jint},
    JNIEnv, JavaVM,
};

#[cfg(target_os = "android")]
use ndk::context::AndroidContext;

#[cfg(target_os = "android")]
use std::sync::OnceLock;

#[cfg(target_os = "android")]
static JAVA_VM: OnceLock<JavaVM> = OnceLock::new();

#[cfg(target_os = "android")]
static ANDROID_CONTEXT: OnceLock<AndroidContext> = OnceLock::new();

/// Android permission constants
#[cfg(target_os = "android")]
const PERMISSION_RECORD_AUDIO: &str = "android.permission.RECORD_AUDIO";
#[cfg(target_os = "android")]
const PERMISSION_INTERNET: &str = "android.permission.INTERNET";
#[cfg(target_os = "android")]
const PERMISSION_GRANTED: i32 = 0;
#[cfg(target_os = "android")]
const REQUEST_CODE_AUDIO: i32 = 1001;

/// Initialize Android context and JavaVM
/// This should be called from the Android activity onCreate
#[cfg(target_os = "android")]
pub fn initialize_android_context(vm: JavaVM, context: AndroidContext) {
    JAVA_VM.get_or_init(|| vm);
    ANDROID_CONTEXT.get_or_init(|| context);
    info!("Android context initialized");
}

/// Request Android audio permissions using JNI
pub fn request_audio_permissions() -> Result<(), AudioError> {
    #[cfg(target_os = "android")]
    {
        info!("Requesting Android audio permissions");
        
        let vm = JAVA_VM.get().ok_or_else(|| {
            error!("JavaVM not initialized");
            AudioError::PermissionDenied
        })?;
        
        let mut env = vm.attach_current_thread().map_err(|e| {
            error!("Failed to attach to JVM thread: {:?}", e);
            AudioError::PermissionDenied
        })?;
        
        // Get the activity context
        let context = get_activity_context(&mut env)?;
        
        // Check if permission is already granted
        if check_permission_internal(&mut env, &context, PERMISSION_RECORD_AUDIO)? {
            info!("Audio permission already granted");
            return Ok(());
        }
        
        // Request permission
        request_permission_internal(&mut env, &context, PERMISSION_RECORD_AUDIO, REQUEST_CODE_AUDIO)?;
        
        info!("Audio permission request sent");
        Ok(())
    }
    
    #[cfg(not(target_os = "android"))]
    {
        Err(AudioError::UnsupportedPlatform)
    }
}

/// Check if audio permissions are granted
pub fn check_audio_permissions() -> bool {
    #[cfg(target_os = "android")]
    {
        let vm = match JAVA_VM.get() {
            Some(vm) => vm,
            None => {
                warn!("JavaVM not initialized, cannot check permissions");
                return false;
            }
        };
        
        let mut env = match vm.attach_current_thread() {
            Ok(env) => env,
            Err(e) => {
                error!("Failed to attach to JVM thread: {:?}", e);
                return false;
            }
        };
        
        let context = match get_activity_context(&mut env) {
            Ok(ctx) => ctx,
            Err(e) => {
                error!("Failed to get activity context: {:?}", e);
                return false;
            }
        };
        
        match check_permission_internal(&mut env, &context, PERMISSION_RECORD_AUDIO) {
            Ok(granted) => granted,
            Err(e) => {
                error!("Failed to check permission: {:?}", e);
                false
            }
        }
    }
    
    #[cfg(not(target_os = "android"))]
    {
        false
    }
}

/// Check if the app should show permission rationale
#[cfg(target_os = "android")]
pub fn should_show_permission_rationale() -> Result<bool, AudioError> {
    let vm = JAVA_VM.get().ok_or(AudioError::PermissionDenied)?;
    let mut env = vm.attach_current_thread().map_err(|_| AudioError::PermissionDenied)?;
    
    let activity = get_activity_context(&mut env)?;
    
    let permission_str = env.new_string(PERMISSION_RECORD_AUDIO)
        .map_err(|_| AudioError::PermissionDenied)?;
    
    let should_show = env.call_method(
        &activity,
        "shouldShowRequestPermissionRationale",
        "(Ljava/lang/String;)Z",
        &[JValue::Object(&permission_str)],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    Ok(should_show.z().unwrap_or(false))
}

/// Open app settings for manual permission grant
#[cfg(target_os = "android")]
pub fn open_app_settings() -> Result<(), AudioError> {
    let vm = JAVA_VM.get().ok_or(AudioError::PermissionDenied)?;
    let mut env = vm.attach_current_thread().map_err(|_| AudioError::PermissionDenied)?;
    
    let context = get_activity_context(&mut env)?;
    
    // Create Intent to open app settings
    let intent_class = env.find_class("android/content/Intent")
        .map_err(|_| AudioError::PermissionDenied)?;
    
    let action_settings = env.new_string("android.settings.APPLICATION_DETAILS_SETTINGS")
        .map_err(|_| AudioError::PermissionDenied)?;
    
    let intent = env.new_object(
        intent_class,
        "(Ljava/lang/String;)V",
        &[JValue::Object(&action_settings)],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    // Start activity
    env.call_method(
        &context,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    info!("Opened app settings");
    Ok(())
}

/// Configure battery optimization exemption
#[cfg(target_os = "android")]
pub fn request_battery_optimization_exemption() -> Result<(), AudioError> {
    let vm = JAVA_VM.get().ok_or(AudioError::PermissionDenied)?;
    let mut env = vm.attach_current_thread().map_err(|_| AudioError::PermissionDenied)?;
    
    let context = get_activity_context(&mut env)?;
    
    // Get PowerManager
    let power_service = env.new_string("power")
        .map_err(|_| AudioError::PermissionDenied)?;
    
    let power_manager = env.call_method(
        &context,
        "getSystemService",
        "(Ljava/lang/String;)Ljava/lang/Object;",
        &[JValue::Object(&power_service)],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    let power_manager = power_manager.l().map_err(|_| AudioError::PermissionDenied)?;
    
    // Check if already ignoring battery optimizations
    let package_name = get_package_name(&mut env, &context)?;
    
    let is_ignoring = env.call_method(
        &power_manager,
        "isIgnoringBatteryOptimizations",
        "(Ljava/lang/String;)Z",
        &[JValue::Object(&package_name)],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    if is_ignoring.z().unwrap_or(false) {
        info!("App already exempt from battery optimization");
        return Ok(());
    }
    
    // Request exemption via Intent
    let intent_class = env.find_class("android/content/Intent")
        .map_err(|_| AudioError::PermissionDenied)?;
    
    let action = env.new_string("android.settings.REQUEST_IGNORE_BATTERY_OPTIMIZATIONS")
        .map_err(|_| AudioError::PermissionDenied)?;
    
    let intent = env.new_object(
        intent_class,
        "(Ljava/lang/String;)V",
        &[JValue::Object(&action)],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    env.call_method(
        &context,
        "startActivity",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    info!("Requested battery optimization exemption");
    Ok(())
}

/// Check if background restrictions are enabled
#[cfg(target_os = "android")]
pub fn check_background_restrictions() -> Result<bool, AudioError> {
    let vm = JAVA_VM.get().ok_or(AudioError::PermissionDenied)?;
    let mut env = vm.attach_current_thread().map_err(|_| AudioError::PermissionDenied)?;
    
    let context = get_activity_context(&mut env)?;
    
    // Get ActivityManager
    let activity_service = env.new_string("activity")
        .map_err(|_| AudioError::PermissionDenied)?;
    
    let activity_manager = env.call_method(
        &context,
        "getSystemService",
        "(Ljava/lang/String;)Ljava/lang/Object;",
        &[JValue::Object(&activity_service)],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    let activity_manager = activity_manager.l().map_err(|_| AudioError::PermissionDenied)?;
    
    // Check if background restricted (API 28+)
    let is_restricted = env.call_method(
        &activity_manager,
        "isBackgroundRestricted",
        "()Z",
        &[],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    Ok(is_restricted.z().unwrap_or(false))
}

// Internal helper functions

#[cfg(target_os = "android")]
fn get_activity_context(env: &mut JNIEnv) -> Result<JObject, AudioError> {
    let context = ANDROID_CONTEXT.get().ok_or(AudioError::PermissionDenied)?;
    
    // Get the activity from the context
    let activity_class = env.find_class("android/app/Activity")
        .map_err(|_| AudioError::PermissionDenied)?;
    
    // Return the context as JObject
    Ok(unsafe { JObject::from_raw(context.context() as *mut _) })
}

#[cfg(target_os = "android")]
fn check_permission_internal(
    env: &mut JNIEnv,
    context: &JObject,
    permission: &str,
) -> Result<bool, AudioError> {
    let permission_str = env.new_string(permission)
        .map_err(|_| AudioError::PermissionDenied)?;
    
    let result = env.call_method(
        context,
        "checkSelfPermission",
        "(Ljava/lang/String;)I",
        &[JValue::Object(&permission_str)],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    let permission_status = result.i().unwrap_or(-1);
    Ok(permission_status == PERMISSION_GRANTED)
}

#[cfg(target_os = "android")]
fn request_permission_internal(
    env: &mut JNIEnv,
    context: &JObject,
    permission: &str,
    request_code: i32,
) -> Result<(), AudioError> {
    let permission_str = env.new_string(permission)
        .map_err(|_| AudioError::PermissionDenied)?;
    
    // Create String array with single permission
    let string_class = env.find_class("java/lang/String")
        .map_err(|_| AudioError::PermissionDenied)?;
    
    let permissions_array = env.new_object_array(1, string_class, permission_str)
        .map_err(|_| AudioError::PermissionDenied)?;
    
    // Request permissions
    env.call_method(
        context,
        "requestPermissions",
        "([Ljava/lang/String;I)V",
        &[
            JValue::Object(&permissions_array),
            JValue::Int(request_code),
        ],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    Ok(())
}

#[cfg(target_os = "android")]
fn get_package_name(env: &mut JNIEnv, context: &JObject) -> Result<JString, AudioError> {
    let package_name = env.call_method(
        context,
        "getPackageName",
        "()Ljava/lang/String;",
        &[],
    ).map_err(|_| AudioError::PermissionDenied)?;
    
    Ok(package_name.l().map_err(|_| AudioError::PermissionDenied)?.into())
}

/// Handle permission request result (called from Android activity)
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_dioxus_voiceassistant_MainActivity_onPermissionResult(
    env: JNIEnv,
    _class: JClass,
    request_code: jint,
    granted: jboolean,
) {
    if request_code == REQUEST_CODE_AUDIO {
        if granted != 0 {
            info!("Audio permission granted");
        } else {
            warn!("Audio permission denied");
        }
    }
}
