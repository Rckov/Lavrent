#ifndef ENGINE_H
#define ENGINE_H

#include <stddef.h>
#include <stdint.h>

/*
 * Document engine C ABI version 1.
 *
 * A Engine pointer and all handles created through it belong to the thread
 * that created the core. String views contain borrowed UTF-8 bytes and remain
 * valid only for the call. Other input pointers must reference aligned storage
 * for the declared element count. Output pointers must reference writable
 * storage for one declared value.
 *
 * Zero is not a valid handle. Functions use it only for absent optional
 * handles. A nonzero status leaves a UTF-8 message in thread-local storage;
 * engine_last_error_message returns an owned copy. The next
 * status-returning call on that thread replaces or clears the stored message.
 * Status functions leave output storage unchanged on failure. A panic status
 * poisons the engine; only engine_free may be called afterward. On the owning
 * thread, engine_free consumes its non-null pointer regardless of status.
 *
 * EngineBytes and returned arrays remain owned by the native library.
 * Release them with the matching free function exactly once. Unused
 * EngineInputEvent fields are ignored and should be zero-initialized by
 * the caller.
 */

#ifdef __cplusplus
extern "C" {
#endif

#define ENGINE_ABI_VERSION 1u

typedef uint32_t EngineStatus;

#define ENGINE_STATUS_OK 0u
#define ENGINE_STATUS_INVALID_ARGUMENT 1u
#define ENGINE_STATUS_WRONG_THREAD 2u
#define ENGINE_STATUS_INVALID_HANDLE 3u
#define ENGINE_STATUS_STALE_HANDLE 4u
#define ENGINE_STATUS_FOREIGN_HANDLE 5u
#define ENGINE_STATUS_CROSS_DOCUMENT_HANDLE 6u
#define ENGINE_STATUS_DESTROYED_HANDLE 7u
#define ENGINE_STATUS_INVALID_MUTATION 8u
#define ENGINE_STATUS_UNSUPPORTED_RESOURCE_SCHEME 9u
#define ENGINE_STATUS_RESOURCE_LIMIT 10u
#define ENGINE_STATUS_RESOURCE_DECODE 11u
#define ENGINE_STATUS_UNRESOLVED_DOCUMENT 12u
#define ENGINE_STATUS_DOCUMENT_FAILURE 13u
#define ENGINE_STATUS_ADAPTER_UNAVAILABLE 14u
#define ENGINE_STATUS_DEVICE_CREATION 15u
#define ENGINE_STATUS_GPU_EXECUTION 16u
#define ENGINE_STATUS_GPU_READBACK 17u
#define ENGINE_STATUS_DEVICE_LOST 18u
#define ENGINE_STATUS_UNSUPPORTED_OPERATION 19u
#define ENGINE_STATUS_PANIC 20u

#define ENGINE_POWER_LOW 0u
#define ENGINE_POWER_HIGH 1u

#define ENGINE_COLOR_LIGHT 0u
#define ENGINE_COLOR_DARK 1u

#define ENGINE_RESOURCE_AUTHOR_STYLESHEET 0u
#define ENGINE_RESOURCE_RASTER_IMAGE 1u
#define ENGINE_RESOURCE_SVG_DOCUMENT 2u
#define ENGINE_RESOURCE_FONT 3u

#define ENGINE_INPUT_POINTER_MOVE 0u
#define ENGINE_INPUT_POINTER_DOWN 1u
#define ENGINE_INPUT_POINTER_UP 2u
#define ENGINE_INPUT_POINTER_CANCEL 3u
#define ENGINE_INPUT_POINTER_LEAVE 4u
#define ENGINE_INPUT_WHEEL 5u
#define ENGINE_INPUT_KEY_DOWN 6u
#define ENGINE_INPUT_KEY_UP 7u
#define ENGINE_INPUT_TEXT 8u
#define ENGINE_INPUT_IME_PREEDIT 9u
#define ENGINE_INPUT_IME_COMMIT 10u
#define ENGINE_INPUT_IME_CANCEL 11u
#define ENGINE_INPUT_FOCUS_GAINED 12u
#define ENGINE_INPUT_FOCUS_LOST 13u

#define ENGINE_POINTER_MOUSE 0u
#define ENGINE_POINTER_TOUCH 1u
#define ENGINE_POINTER_PEN 2u

#define ENGINE_POINTER_BUTTON_PRIMARY 0u
#define ENGINE_POINTER_BUTTON_SECONDARY 1u
#define ENGINE_POINTER_BUTTON_AUXILIARY 2u
#define ENGINE_POINTER_BUTTON_BACK 3u
#define ENGINE_POINTER_BUTTON_FORWARD 4u

#define ENGINE_POINTER_BUTTONS_PRIMARY (1u << 0)
#define ENGINE_POINTER_BUTTONS_SECONDARY (1u << 1)
#define ENGINE_POINTER_BUTTONS_AUXILIARY (1u << 2)
#define ENGINE_POINTER_BUTTONS_BACK (1u << 3)
#define ENGINE_POINTER_BUTTONS_FORWARD (1u << 4)

#define ENGINE_MODIFIER_SHIFT (1u << 0)
#define ENGINE_MODIFIER_CONTROL (1u << 1)
#define ENGINE_MODIFIER_ALT (1u << 2)
#define ENGINE_MODIFIER_META (1u << 3)

#define ENGINE_KEY_UNIDENTIFIED 0u
#define ENGINE_KEY_NAMED 1u
#define ENGINE_KEY_CHARACTER 2u

#define ENGINE_NAMED_KEY_ALT 0u
#define ENGINE_NAMED_KEY_ARROW_DOWN 1u
#define ENGINE_NAMED_KEY_ARROW_LEFT 2u
#define ENGINE_NAMED_KEY_ARROW_RIGHT 3u
#define ENGINE_NAMED_KEY_ARROW_UP 4u
#define ENGINE_NAMED_KEY_BACKSPACE 5u
#define ENGINE_NAMED_KEY_CONTROL 6u
#define ENGINE_NAMED_KEY_DELETE 7u
#define ENGINE_NAMED_KEY_END 8u
#define ENGINE_NAMED_KEY_ENTER 9u
#define ENGINE_NAMED_KEY_ESCAPE 10u
#define ENGINE_NAMED_KEY_HOME 11u
#define ENGINE_NAMED_KEY_INSERT 12u
#define ENGINE_NAMED_KEY_META 13u
#define ENGINE_NAMED_KEY_PAGE_DOWN 14u
#define ENGINE_NAMED_KEY_PAGE_UP 15u
#define ENGINE_NAMED_KEY_SHIFT 16u
#define ENGINE_NAMED_KEY_SPACE 17u
#define ENGINE_NAMED_KEY_TAB 18u

#define ENGINE_KEY_LOCATION_STANDARD 0u
#define ENGINE_KEY_LOCATION_LEFT 1u
#define ENGINE_KEY_LOCATION_RIGHT 2u
#define ENGINE_KEY_LOCATION_NUMPAD 3u

#define ENGINE_UI_EVENT_CLICK 0u
#define ENGINE_UI_EVENT_INPUT 1u
#define ENGINE_UI_EVENT_CHANGE 2u
#define ENGINE_UI_EVENT_SUBMIT 3u
#define ENGINE_UI_EVENT_FOCUS 4u
#define ENGINE_UI_EVENT_BLUR 5u

#define ENGINE_UI_DATA_NONE 0u
#define ENGINE_UI_DATA_TEXT 1u

#define ENGINE_DIAGNOSTIC_INFORMATION 0u
#define ENGINE_DIAGNOSTIC_WARNING 1u
#define ENGINE_DIAGNOSTIC_ERROR 2u

#define ENGINE_DIAGNOSTIC_RENDERER_INITIALIZED 0u
#define ENGINE_DIAGNOSTIC_MISSING_RESOURCE 1u
#define ENGINE_DIAGNOSTIC_RESOURCE_DECODE_FAILED 2u
#define ENGINE_DIAGNOSTIC_RESOURCE_LIMIT_EXCEEDED 3u
#define ENGINE_DIAGNOSTIC_UNSUPPORTED_PAINT 4u
#define ENGINE_DIAGNOSTIC_UNSUPPORTED_OPERATION 5u
#define ENGINE_DIAGNOSTIC_CAPABILITY_UNAVAILABLE 6u
#define ENGINE_DIAGNOSTIC_DEVICE_LOST 7u

typedef struct Engine Engine;

typedef struct EngineBytes {
    uint8_t *data;
    size_t length;
} EngineBytes;

typedef struct EngineOptions {
    uint32_t power_preference;
} EngineOptions;

typedef struct EngineStringView {
    const uint8_t *data;
    size_t length;
} EngineStringView;

typedef struct EngineDocumentSource {
    EngineStringView html;
    const EngineStringView *author_css;
    size_t author_css_length;
    EngineStringView base_resource_uri;
} EngineDocumentSource;

typedef struct EngineViewport {
    double width;
    double height;
    double scale_factor;
    uint32_t color_scheme;
} EngineViewport;

typedef struct EnginePoint {
    double x;
    double y;
} EnginePoint;

typedef struct EngineRect {
    double x;
    double y;
    double width;
    double height;
} EngineRect;

typedef struct EngineHandleArray {
    uint64_t *data;
    size_t length;
} EngineHandleArray;

typedef struct EngineResourceSource {
    EngineStringView uri;
    EngineStringView media_type;
    uint32_t kind;
    const uint8_t *data;
    size_t length;
} EngineResourceSource;

typedef struct EngineRenderResult {
    uint32_t width;
    uint32_t height;
} EngineRenderResult;

typedef struct EngineReadback {
    uint32_t width;
    uint32_t height;
    EngineBytes pixels;
} EngineReadback;

typedef struct EngineInputEvent {
    uint32_t kind;
    uint64_t pointer_id;
    uint32_t pointer_kind;
    uint8_t is_primary;
    EnginePoint position;
    uint32_t button;
    uint32_t buttons;
    uint32_t modifiers;
    double delta_x;
    double delta_y;
    uint32_t key_kind;
    uint32_t named_key;
    EngineStringView key_text;
    EngineStringView physical_key;
    uint32_t key_location;
    uint8_t repeat;
    EngineStringView text;
    uint8_t has_ime_selection;
    size_t ime_selection_start;
    size_t ime_selection_end;
} EngineInputEvent;

typedef struct EngineUiEvent {
    uint64_t target;
    uint32_t kind;
    uint32_t data_kind;
    EngineBytes text;
} EngineUiEvent;

typedef struct EngineUiEventArray {
    EngineUiEvent *data;
    size_t length;
} EngineUiEventArray;

typedef struct EngineDiagnosticContext {
    EngineBytes key;
    EngineBytes value;
} EngineDiagnosticContext;

typedef struct EngineDiagnostic {
    uint32_t code;
    uint32_t severity;
    uint64_t document;
    uint64_t node;
    EngineBytes message;
    EngineDiagnosticContext *context;
    size_t context_length;
} EngineDiagnostic;

typedef struct EngineDiagnosticArray {
    EngineDiagnostic *data;
    size_t length;
} EngineDiagnosticArray;

uint32_t engine_abi_version(void);
EngineBytes engine_last_error_message(void);
void engine_bytes_free(EngineBytes bytes);
void engine_handles_free(EngineHandleArray handles);
EngineStatus engine_new(EngineOptions options, Engine **output);
EngineStatus engine_free(Engine *core);
EngineStatus engine_document_create(
    Engine *core,
    EngineDocumentSource source,
    EngineViewport viewport,
    uint64_t *output);
EngineStatus engine_document_destroy(Engine *core, uint64_t document);
EngineStatus engine_document_replace_source(
    Engine *core,
    uint64_t document,
    EngineDocumentSource source);
EngineStatus engine_document_find_element_by_id(
    Engine *core,
    uint64_t document,
    EngineStringView id,
    uint64_t *output);
EngineStatus engine_node_create_element(
    Engine *core,
    uint64_t document,
    EngineStringView name,
    uint64_t *output);
EngineStatus engine_node_create_text(
    Engine *core,
    uint64_t document,
    EngineStringView text,
    uint64_t *output);
EngineStatus engine_node_append(
    Engine *core,
    uint64_t document,
    uint64_t parent,
    uint64_t child);
EngineStatus engine_node_insert_before(
    Engine *core,
    uint64_t document,
    uint64_t anchor,
    uint64_t node);
EngineStatus engine_node_move(
    Engine *core,
    uint64_t document,
    uint64_t node,
    uint64_t new_parent,
    uint64_t before);
EngineStatus engine_node_remove(Engine *core, uint64_t document, uint64_t node);
EngineStatus engine_node_set_attribute(
    Engine *core,
    uint64_t document,
    uint64_t node,
    EngineStringView name,
    EngineStringView value);
EngineStatus engine_node_remove_attribute(
    Engine *core,
    uint64_t document,
    uint64_t node,
    EngineStringView name);
EngineStatus engine_node_replace_text(
    Engine *core,
    uint64_t document,
    uint64_t node,
    EngineStringView text);
EngineStatus engine_document_set_viewport(
    Engine *core,
    uint64_t document,
    EngineViewport viewport);
EngineStatus engine_document_resolve(
    Engine *core,
    uint64_t document,
    uint64_t animation_time_nanoseconds);
EngineStatus engine_node_border_box(
    Engine *core,
    uint64_t document,
    uint64_t node,
    EngineRect *output);
EngineStatus engine_document_hit_test(
    Engine *core,
    uint64_t document,
    EnginePoint point,
    EngineHandleArray *output);
EngineStatus engine_resource_register(
    Engine *core,
    EngineResourceSource source,
    uint64_t *output);
EngineStatus engine_resource_replace(
    Engine *core,
    uint64_t resource,
    EngineResourceSource source);
EngineStatus engine_resource_remove(Engine *core, uint64_t resource);
EngineStatus engine_render_target_create(
    Engine *core,
    uint32_t width,
    uint32_t height,
    uint64_t *output);
EngineStatus engine_render_target_resize(
    Engine *core,
    uint64_t target,
    uint32_t width,
    uint32_t height);
EngineStatus engine_render_target_destroy(Engine *core, uint64_t target);
EngineStatus engine_render(
    Engine *core,
    uint64_t document,
    uint64_t target,
    EngineRenderResult *output);
EngineStatus engine_render_target_readback_rgba8(
    Engine *core,
    uint64_t target,
    EngineReadback *output);
void engine_readback_free(EngineReadback readback);
EngineStatus engine_input_dispatch(
    Engine *core,
    uint64_t document,
    EngineInputEvent event);
EngineStatus engine_ui_events_drain(Engine *core, EngineUiEventArray *output);
void engine_ui_events_free(EngineUiEventArray events);
EngineStatus engine_diagnostics_drain(
    Engine *core,
    EngineDiagnosticArray *output);
void engine_diagnostics_free(EngineDiagnosticArray diagnostics);

#ifdef __cplusplus
}
#endif

#endif
