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
        println!("cargo:rustc-link-lib=dylib=avrt"); // For MMCSS
        
        // Embed Windows resources (icon and manifest)
        if std::path::Path::new("windows/resource.rc").exists() {
            embed_resource::compile("windows/resource.rc", embed_resource::NONE);
            println!("cargo:rerun-if-changed=windows/resource.rc");
            println!("cargo:rerun-if-changed=windows/app.manifest");
            
            // Check if icon exists, if not create a placeholder note
            if !std::path::Path::new("windows/icon.ico").exists() {
                eprintln!("Warning: windows/icon.ico not found. Please add an icon file.");
                eprintln!("You can create one using an online tool or image editor.");
            } else {
                println!("cargo:rerun-if-changed=windows/icon.ico");
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=AudioUnit");
    }
}
