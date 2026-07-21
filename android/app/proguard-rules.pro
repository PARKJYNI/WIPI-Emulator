# WipiNative의 external 함수는 JNI(Java_com_parkjeongseop_wipi_WipiNative_*)로 호출되므로 R8이 제거/난독화하면 안 됨
-keep class com.parkjeongseop.wipi.WipiNative { *; }
-keepclasseswithmembernames class * {
    native <methods>;
}
