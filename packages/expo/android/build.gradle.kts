plugins {
    id("expo.modules.kotlin")
}

android {
    namespace = "expo.modules.kimchi"
    compileSdk = 34

    defaultConfig {
        minSdk = 26
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }
}

dependencies {
    // Option 1: Use local project dependency during development
    // implementation(project(":kimchi-mobile-kotlin"))

    // Option 2: Use published Maven artifact
    // implementation("com.kimchi:kimchi-mobile:1.0.0")

    // For local development, include the AAR directly
    // The path is relative to this build.gradle.kts
    implementation(files("../kotlin/build/outputs/aar/kimchi-mobile-release.aar"))
}
