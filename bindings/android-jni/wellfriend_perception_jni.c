/*
 * Deliberately thin Android JNI transport for the audited Wellfriend C ABI.
 * It owns no perception algorithm and returns the JSON allocated by wf_*.
 */
#include <jni.h>
#include <stdint.h>

#include "wellfriend_perception.h"

JNIEXPORT jlong JNICALL Java_dev_wellfriend_scan_perception_JniNativePerceptionBridge_nativeCreate(
    JNIEnv *env, jclass cls, jstring config) {
    (void)cls;
    if (config == NULL) return (jlong)0;
    const char *text = (*env)->GetStringUTFChars(env, config, 0);
    if (text == NULL) return (jlong)0;
    WfEngine *engine = wf_engine_create(text);
    (*env)->ReleaseStringUTFChars(env, config, text);
    return (jlong)(intptr_t)engine;
}

JNIEXPORT void JNICALL Java_dev_wellfriend_scan_perception_JniNativePerceptionBridge_nativeDestroy(
    JNIEnv *env, jclass cls, jlong handle) {
    (void)env;
    (void)cls;
    wf_engine_destroy((WfEngine *)(intptr_t)handle);
}

static jstring invoke(
    JNIEnv *env,
    jlong handle,
    jbyteArray image,
    jint width,
    jint height,
    jint stride,
    jstring format,
    jstring request,
    int operation) {
    if (image == NULL || format == NULL || request == NULL || handle == 0) {
        return (*env)->NewStringUTF(env, "{\"schema_version\":1,\"error\":{\"code\":\"invalid_input\",\"message\":\"null JNI input\"}}");
    }
    jbyte *bytes = (*env)->GetByteArrayElements(env, image, 0);
    const char *format_text = (*env)->GetStringUTFChars(env, format, 0);
    const char *request_text = (*env)->GetStringUTFChars(env, request, 0);
    if (bytes == NULL || format_text == NULL || request_text == NULL) {
        if (request_text != NULL) (*env)->ReleaseStringUTFChars(env, request, request_text);
        if (format_text != NULL) (*env)->ReleaseStringUTFChars(env, format, format_text);
        if (bytes != NULL) (*env)->ReleaseByteArrayElements(env, image, bytes, JNI_ABORT);
        return (*env)->NewStringUTF(env, "{\"schema_version\":1,\"error\":{\"code\":\"invalid_input\",\"message\":\"JNI conversion failed\"}}");
    }
    WfEngine *engine = (WfEngine *)(intptr_t)handle;
    char *response = operation == 0
        ? wf_analyze_frame(engine, (const uint8_t *)bytes, (uint32_t)width, (uint32_t)height, (uint32_t)stride, format_text, request_text)
        : operation == 1
            ? wf_reconstruct_page(engine, (const uint8_t *)bytes, (uint32_t)width, (uint32_t)height, (uint32_t)stride, format_text, request_text)
            : wf_apply_filter(engine, (const uint8_t *)bytes, (uint32_t)width, (uint32_t)height, (uint32_t)stride, format_text, request_text);
    jstring output = (*env)->NewStringUTF(env, response != NULL ? response : "{\"schema_version\":1,\"error\":{\"code\":\"runtime_failure\",\"message\":\"native runtime returned no response\"}}");
    if (response != NULL) wf_string_free(response);
    (*env)->ReleaseStringUTFChars(env, request, request_text);
    (*env)->ReleaseStringUTFChars(env, format, format_text);
    (*env)->ReleaseByteArrayElements(env, image, bytes, JNI_ABORT);
    return output;
}

JNIEXPORT jstring JNICALL Java_dev_wellfriend_scan_perception_JniNativePerceptionBridge_nativeAnalyze(JNIEnv *env, jclass cls, jlong handle, jbyteArray image, jint width, jint height, jint stride, jstring format, jstring request) { (void)cls; return invoke(env, handle, image, width, height, stride, format, request, 0); }
JNIEXPORT jstring JNICALL Java_dev_wellfriend_scan_perception_JniNativePerceptionBridge_nativeReconstruct(JNIEnv *env, jclass cls, jlong handle, jbyteArray image, jint width, jint height, jint stride, jstring format, jstring request) { (void)cls; return invoke(env, handle, image, width, height, stride, format, request, 1); }
JNIEXPORT jstring JNICALL Java_dev_wellfriend_scan_perception_JniNativePerceptionBridge_nativeApplyFilter(JNIEnv *env, jclass cls, jlong handle, jbyteArray image, jint width, jint height, jint stride, jstring format, jstring request) { (void)cls; return invoke(env, handle, image, width, height, stride, format, request, 2); }
