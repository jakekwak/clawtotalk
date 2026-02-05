package com.dioxus.voiceassistant;

import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.os.PowerManager;
import android.provider.Settings;
import android.util.Log;
import androidx.annotation.NonNull;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

/**
 * Main Activity for Dioxus Voice Assistant
 * Handles Android-specific initialization and permission management
 */
public class MainActivity extends Activity {
    private static final String TAG = "VoiceAssistant";
    private static final int REQUEST_RECORD_AUDIO = 1001;
    private static final int REQUEST_BATTERY_OPTIMIZATION = 1002;
    
    // Native library
    static {
        System.loadLibrary("dioxus_voice_assistant");
    }
    
    // Native methods
    private native void initializeNative();
    private native void onPermissionResult(int requestCode, boolean granted);
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        Log.d(TAG, "MainActivity onCreate");
        
        // Initialize native code
        initializeNative();
        
        // Check and request permissions
        checkPermissions();
        
        // Check battery optimization
        checkBatteryOptimization();
    }
    
    /**
     * Check and request necessary permissions
     */
    private void checkPermissions() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            if (ContextCompat.checkSelfPermission(this, Manifest.permission.RECORD_AUDIO)
                    != PackageManager.PERMISSION_GRANTED) {
                
                // Should we show an explanation?
                if (ActivityCompat.shouldShowRequestPermissionRationale(this,
                        Manifest.permission.RECORD_AUDIO)) {
                    // Show explanation to the user
                    Log.d(TAG, "Showing permission rationale");
                    // In a real app, show a dialog explaining why the permission is needed
                }
                
                // Request the permission
                ActivityCompat.requestPermissions(this,
                        new String[]{Manifest.permission.RECORD_AUDIO},
                        REQUEST_RECORD_AUDIO);
            } else {
                Log.d(TAG, "Audio permission already granted");
            }
        }
    }
    
    /**
     * Check battery optimization status and request exemption if needed
     */
    private void checkBatteryOptimization() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            PowerManager pm = (PowerManager) getSystemService(POWER_SERVICE);
            String packageName = getPackageName();
            
            if (!pm.isIgnoringBatteryOptimizations(packageName)) {
                Log.d(TAG, "App is subject to battery optimization");
                // In a real app, show a dialog explaining why exemption is beneficial
                // Then call requestBatteryOptimizationExemption() if user agrees
            } else {
                Log.d(TAG, "App is exempt from battery optimization");
            }
        }
    }
    
    /**
     * Request battery optimization exemption
     */
    public void requestBatteryOptimizationExemption() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            Intent intent = new Intent();
            intent.setAction(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS);
            intent.setData(Uri.parse("package:" + getPackageName()));
            startActivityForResult(intent, REQUEST_BATTERY_OPTIMIZATION);
        }
    }
    
    /**
     * Open app settings for manual permission grant
     */
    public void openAppSettings() {
        Intent intent = new Intent();
        intent.setAction(Settings.ACTION_APPLICATION_DETAILS_SETTINGS);
        Uri uri = Uri.fromParts("package", getPackageName(), null);
        intent.setData(uri);
        startActivity(intent);
    }
    
    @Override
    public void onRequestPermissionsResult(int requestCode, @NonNull String[] permissions,
                                          @NonNull int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        
        if (requestCode == REQUEST_RECORD_AUDIO) {
            boolean granted = grantResults.length > 0 
                && grantResults[0] == PackageManager.PERMISSION_GRANTED;
            
            Log.d(TAG, "Audio permission " + (granted ? "granted" : "denied"));
            
            // Notify native code
            onPermissionResult(requestCode, granted);
            
            if (!granted) {
                // Check if we should show rationale
                if (!ActivityCompat.shouldShowRequestPermissionRationale(this,
                        Manifest.permission.RECORD_AUDIO)) {
                    // User selected "Don't ask again"
                    Log.d(TAG, "Permission permanently denied");
                    // In a real app, show dialog with option to open settings
                }
            }
        }
    }
    
    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        
        if (requestCode == REQUEST_BATTERY_OPTIMIZATION) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                PowerManager pm = (PowerManager) getSystemService(POWER_SERVICE);
                boolean isIgnoring = pm.isIgnoringBatteryOptimizations(getPackageName());
                Log.d(TAG, "Battery optimization exemption " + 
                    (isIgnoring ? "granted" : "denied"));
            }
        }
    }
    
    @Override
    protected void onResume() {
        super.onResume();
        Log.d(TAG, "MainActivity onResume");
    }
    
    @Override
    protected void onPause() {
        super.onPause();
        Log.d(TAG, "MainActivity onPause");
    }
    
    @Override
    protected void onDestroy() {
        super.onDestroy();
        Log.d(TAG, "MainActivity onDestroy");
    }
}
