# Consumer ProGuard rules for kimchi-mobile
# These rules are applied to consumers of this library

# Keep all public classes and methods in the kimchi.mobile package
-keep class com.kimchi.mobile.** { *; }

# Keep JNA classes needed for native library loading
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
