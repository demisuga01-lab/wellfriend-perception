#ifndef WELLFRIEND_PERCEPTION_H
#define WELLFRIEND_PERCEPTION_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct WfEngine WfEngine;
WfEngine *wf_engine_create(const char *config_json);
void wf_engine_destroy(WfEngine *engine);
char *wf_analyze_frame(WfEngine *engine, const uint8_t *image_bytes, uint32_t width, uint32_t height, uint32_t stride, const char *pixel_format, const char *request_json);
char *wf_reconstruct_page(WfEngine *engine, const uint8_t *image_bytes, uint32_t width, uint32_t height, uint32_t stride, const char *pixel_format, const char *request_json);
char *wf_apply_filter(WfEngine *engine, const uint8_t *image_bytes, uint32_t width, uint32_t height, uint32_t stride, const char *pixel_format, const char *request_json);
void wf_string_free(char *pointer);
const char *wf_last_error(const WfEngine *engine);
const char *wf_version(void);

#ifdef __cplusplus
}
#endif
#endif
