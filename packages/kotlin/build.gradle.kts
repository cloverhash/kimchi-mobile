plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("maven-publish")
}

android {
    namespace = "com.kimchi.mobile"
    compileSdk = 34

    defaultConfig {
        minSdk = 26

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")

        ndk {
            // Supported ABIs
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.12.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.3")

    // JNA for UniFFI generated bindings (Android AAR version)
    implementation("net.java.dev.jna:jna:5.14.0@aar")

    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.5")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.1")
}

// Task to build native library
tasks.register<Exec>("buildNativeLibrary") {
    workingDir = file("../../")
    commandLine("./scripts/build-android.sh")
}

// Task to generate UniFFI bindings
tasks.register<Exec>("generateBindings") {
    dependsOn("buildNativeLibrary")
    workingDir = file("../../")
    commandLine(
        "cargo", "run", "-p", "uniffi-bindgen",
        "generate", "kimchi-ffi/src/kimchi_ffi.udl",
        "--language", "kotlin",
        "--out-dir", "packages/kotlin/src/main/kotlin"
    )
}

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = "com.kimchi"
            artifactId = "kimchi-mobile"
            version = "0.1.0"

            afterEvaluate {
                from(components["release"])
            }
        }
    }
}
