#pragma once
#include <stdint.h>

#if defined(_WIN32)
#define TEKTALK_PLUGIN_EXPORT __declspec(dllexport)
#else
#define TEKTALK_PLUGIN_EXPORT __attribute__((visibility("default")))
#endif

#ifdef __cplusplus
extern "C" {
#endif

#define TEKTALK_DESKTOP_PLUGIN_ABI 1u

typedef struct tektalk_host_api {
  uint32_t abi_version;
  void* context;
  int32_t (*invoke)(void* context, const char* capability,
                    const uint8_t* request, uintptr_t request_length,
                    uint8_t** response, uintptr_t* response_length);
  void (*release_response)(void* context, uint8_t* response,
                           uintptr_t response_length);
} tektalk_host_api;

TEKTALK_PLUGIN_EXPORT uint32_t tektalk_plugin_abi_version(void);
TEKTALK_PLUGIN_EXPORT const char* tektalk_plugin_id(void);
TEKTALK_PLUGIN_EXPORT int32_t tektalk_plugin_health_check(void);
TEKTALK_PLUGIN_EXPORT void* tektalk_plugin_create_view(const tektalk_host_api* host);
TEKTALK_PLUGIN_EXPORT void tektalk_plugin_destroy_view(void* view);

#ifdef __cplusplus
}
#endif
