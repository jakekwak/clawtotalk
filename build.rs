fn main() {
    // Platform-specific build configuration
    
    #[cfg(target_os = "android")]
    {
        println!("cargo:rustc-link-lib=dylib=OpenSLES");
        println!("cargo:rustc-link-lib=dylib=log");
    }
    
    #[cfg(target_os = "ios")]
    {
        println!("cargo:rustc-link-lib=framework=AVFoundation");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
    }
    
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=dylib=ole32");
        println!("cargo:rustc-link-lib=dylib=winmm");
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=AudioUnit");
    }
}
