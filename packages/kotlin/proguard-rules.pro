# ProGuard rules for kimchi-mobile library

# Keep JNA classes
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }

# Keep native library loading
-keepclassmembers class * {
    native <methods>;
}

# Keep kimchi-mobile public API
-keep class com.kimchi.mobile.** { *; }
