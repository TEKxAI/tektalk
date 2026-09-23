plugins { id("com.android.application"); id("org.jetbrains.kotlin.android"); id("org.jetbrains.kotlin.plugin.serialization") }
android { namespace="vn.tektalk"; compileSdk=35
    defaultConfig { applicationId="vn.tektalk"; minSdk=28; targetSdk=35; versionCode=1; versionName="0.1"; buildConfigField("String","API_BASE_URL","\"http://10.0.2.2:8080\"") }
    buildFeatures { compose=true; buildConfig=true }
}
dependencies {
    implementation(platform("androidx.compose:compose-bom:2024.11.00")); implementation("androidx.activity:activity-compose:1.9.3"); implementation("androidx.compose.material3:material3")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.7.3"); implementation("com.squareup.okhttp3:okhttp:4.12.0")
    implementation("org.bouncycastle:bcprov-jdk18on:1.79")
}
