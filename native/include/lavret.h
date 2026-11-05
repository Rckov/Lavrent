#ifndef LAVRET_H
#define LAVRET_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define LAVRET_ABI_VERSION 1u

typedef uint32_t LavretStatus;

#define LAVRET_STATUS_OK 0u
#define LAVRET_STATUS_INVALID_ARGUMENT 1u
#define LAVRET_STATUS_WRONG_THREAD 2u
#define LAVRET_STATUS_INVALID_HANDLE 3u
#define LAVRET_STATUS_STALE_HANDLE 4u
#define LAVRET_STATUS_FOREIGN_HANDLE 5u
#define LAVRET_STATUS_CROSS_DOCUMENT_HANDLE 6u
#define LAVRET_STATUS_DESTROYED_HANDLE 7u
#define LAVRET_STATUS_INVALID_MUTATION 8u
#define LAVRET_STATUS_UNSUPPORTED_RESOURCE_SCHEME 9u
#define LAVRET_STATUS_RESOURCE_LIMIT 10u
#define LAVRET_STATUS_RESOURCE_DECODE 11u
#define LAVRET_STATUS_UNRESOLVED_DOCUMENT 12u
#define LAVRET_STATUS_DOCUMENT_FAILURE 13u
#define LAVRET_STATUS_ADAPTER_UNAVAILABLE 14u
#define LAVRET_STATUS_DEVICE_CREATION 15u
#define LAVRET_STATUS_GPU_EXECUTION 16u
#define LAVRET_STATUS_GPU_READBACK 17u
#define LAVRET_STATUS_DEVICE_LOST 18u
#define LAVRET_STATUS_UNSUPPORTED_OPERATION 19u
#define LAVRET_STATUS_PANIC 20u

#define LAVRET_POWER_LOW 0u
#define LAVRET_POWER_HIGH 1u

#define LAVRET_COLOR_LIGHT 0u
#define LAVRET_COLOR_DARK 1u

#define LAVRET_RESOURCE_AUTHOR_STYLESHEET 0u
#define LAVRET_RESOURCE_RASTER_IMAGE 1u
#define LAVRET_RESOURCE_SVG_DOCUMENT 2u
#define LAVRET_RESOURCE_FONT 3u

#define LAVRET_INPUT_POINTER_MOVE 0u
#define LAVRET_INPUT_POINTER_DOWN 1u
#define LAVRET_INPUT_POINTER_UP 2u
#define LAVRET_INPUT_POINTER_CANCEL 3u
#define LAVRET_INPUT_POINTER_LEAVE 4u
#define LAVRET_INPUT_WHEEL 5u
#define LAVRET_INPUT_KEY_DOWN 6u
#define LAVRET_INPUT_KEY_UP 7u
#define LAVRET_INPUT_TEXT 8u
#define LAVRET_INPUT_IME_PREEDIT 9u
#define LAVRET_INPUT_IME_COMMIT 10u
#define LAVRET_INPUT_IME_CANCEL 11u
#define LAVRET_INPUT_FOCUS_GAINED 12u
#define LAVRET_INPUT_FOCUS_LOST 13u

#define LAVRET_POINTER_MOUSE 0u
#define LAVRET_POINTER_TOUCH 1u
#define LAVRET_POINTER_PEN 2u

#define LAVRET_POINTER_BUTTON_PRIMARY 0u
#define LAVRET_POINTER_BUTTON_SECONDARY 1u
#define LAVRET_POINTER_BUTTON_AUXILIARY 2u
#define LAVRET_POINTER_BUTTON_BACK 3u
#define LAVRET_POINTER_BUTTON_FORWARD 4u

#define LAVRET_POINTER_BUTTONS_PRIMARY (1u << 0)
#define LAVRET_POINTER_BUTTONS_SECONDARY (1u << 1)
#define LAVRET_POINTER_BUTTONS_AUXILIARY (1u << 2)
#define LAVRET_POINTER_BUTTONS_BACK (1u << 3)
#define LAVRET_POINTER_BUTTONS_FORWARD (1u << 4)

#define LAVRET_MODIFIER_SHIFT (1u << 0)
#define LAVRET_MODIFIER_CONTROL (1u << 1)
#define LAVRET_MODIFIER_ALT (1u << 2)
#define LAVRET_MODIFIER_META (1u << 3)

#define LAVRET_KEY_UNIDENTIFIED 0u
#define LAVRET_KEY_NAMED 1u
#define LAVRET_KEY_CHARACTER 2u

#define LAVRET_NAMED_KEY_ALT 0u
#define LAVRET_NAMED_KEY_ARROW_DOWN 1u
#define LAVRET_NAMED_KEY_ARROW_LEFT 2u
#define LAVRET_NAMED_KEY_ARROW_RIGHT 3u
#define LAVRET_NAMED_KEY_ARROW_UP 4u
#define LAVRET_NAMED_KEY_BACKSPACE 5u
#define LAVRET_NAMED_KEY_CONTROL 6u
#define LAVRET_NAMED_KEY_DELETE 7u
#define LAVRET_NAMED_KEY_END 8u
#define LAVRET_NAMED_KEY_ENTER 9u
#define LAVRET_NAMED_KEY_ESCAPE 10u
#define LAVRET_NAMED_KEY_HOME 11u
#define LAVRET_NAMED_KEY_INSERT 12u
#define LAVRET_NAMED_KEY_META 13u
#define LAVRET_NAMED_KEY_PAGE_DOWN 14u
#define LAVRET_NAMED_KEY_PAGE_UP 15u
#define LAVRET_NAMED_KEY_SHIFT 16u
#define LAVRET_NAMED_KEY_SPACE 17u
#define LAVRET_NAMED_KEY_TAB 18u

#define LAVRET_KEY_LOCATION_STANDARD 0u
#define LAVRET_KEY_LOCATION_LEFT 1u
#define LAVRET_KEY_LOCATION_RIGHT 2u
#define LAVRET_KEY_LOCATION_NUMPAD 3u

#define LAVRET_UI_EVENT_CLICK 0u
#define LAVRET_UI_EVENT_INPUT 1u
#define LAVRET_UI_EVENT_CHANGE 2u
#define LAVRET_UI_EVENT_SUBMIT 3u
#define LAVRET_UI_EVENT_FOCUS 4u
#define LAVRET_UI_EVENT_BLUR 5u

#define LAVRET_UI_DATA_NONE 0u
#define LAVRET_UI_DATA_TEXT 1u

#define LAVRET_DIAGNOSTIC_INFORMATION 0u
#define LAVRET_DIAGNOSTIC_WARNING 1u
#define LAVRET_DIAGNOSTIC_ERROR 2u

#define LAVRET_DIAGNOSTIC_RENDERER_INITIALIZED 0u
#define LAVRET_DIAGNOSTIC_MISSING_RESOURCE 1u
#define LAVRET_DIAGNOSTIC_RESOURCE_DECODE_FAILED 2u
#define LAVRET_DIAGNOSTIC_RESOURCE_LIMIT_EXCEEDED 3u
#define LAVRET_DIAGNOSTIC_UNSUPPORTED_PAINT 4u
#define LAVRET_DIAGNOSTIC_UNSUPPORTED_OPERATION 5u
#define LAVRET_DIAGNOSTIC_CAPABILITY_UNAVAILABLE 6u
#define LAVRET_DIAGNOSTIC_DEVICE_LOST 7u

typedef struct LavretCore LavretCore;

typedef struct LavretBytes {
    uint8_t *data;
    size_t length;
} LavretBytes;

typedef struct LavretCoreOptions {
    uint32_t power_preference;
} LavretCoreOptions;

typedef struct LavretStringView {
    const uint8_t *data;
    size_t length;
} LavretStringView;

typedef struct LavretDocumentSource {
    LavretStringView html;
    const LavretStringView *author_css;
    size_t author_css_length;
    LavretStringView base_resource_uri;
} LavretDocumentSource;

typedef struct LavretViewport {
    double width;
    double height;
    double scale_factor;
    uint32_t color_scheme;
} LavretViewport;

typedef struct LavretPoint {
    double x;
    double y;
} LavretPoint;

typedef struct LavretRect {
    double x;
    double y;
    double width;
    double height;
} LavretRect;

typedef struct LavretHandleArray {
    uint64_t *data;
    size_t length;
} LavretHandleArray;

typedef struct LavretResourceSource {
    LavretStringView uri;
    LavretStringView media_type;
    uint32_t kind;
    const uint8_t *data;
    size_t length;
} LavretResourceSource;

typedef struct LavretRenderResult {
    uint32_t width;
    uint32_t height;
} LavretRenderResult;

typedef struct LavretReadback {
    uint32_t width;
    uint32_t height;
    LavretBytes pixels;
} LavretReadback;

typedef struct LavretInputEvent {
    uint32_t kind;
    uint64_t pointer_id;
    uint32_t pointer_kind;
    uint8_t is_primary;
    LavretPoint position;
    uint32_t button;
    uint32_t buttons;
    uint32_t modifiers;
    double delta_x;
    double delta_y;
    uint32_t key_kind;
    uint32_t named_key;
    LavretStringView key_text;
    LavretStringView physical_key;
    uint32_t key_location;
    uint8_t repeat;
    LavretStringView text;
    uint8_t has_ime_selection;
    size_t ime_selection_start;
    size_t ime_selection_end;
} LavretInputEvent;

typedef struct LavretUiEvent {
    uint64_t target;
    uint32_t kind;
    uint32_t data_kind;
    LavretBytes text;
} LavretUiEvent;

typedef struct LavretUiEventArray {
    LavretUiEvent *data;
    size_t length;
} LavretUiEventArray;

typedef struct LavretDiagnosticContext {
    LavretBytes key;
    LavretBytes value;
} LavretDiagnosticContext;

typedef struct LavretDiagnostic {
    uint32_t code;
    uint32_t severity;
    uint64_t document;
    uint64_t node;
    LavretBytes message;
    LavretDiagnosticContext *context;
    size_t context_length;
} LavretDiagnostic;

typedef struct LavretDiagnosticArray {
    LavretDiagnostic *data;
    size_t length;
} LavretDiagnosticArray;

uint32_t lavret_abi_version(void);
LavretBytes lavret_last_error_message(void);
void lavret_bytes_free(LavretBytes bytes);
void lavret_handles_free(LavretHandleArray handles);
LavretStatus lavret_core_new(LavretCoreOptions options, LavretCore **output);
LavretStatus lavret_core_free(LavretCore *core);
LavretStatus lavret_document_create(LavretCore *core, LavretDocumentSource source, LavretViewport viewport, uint64_t *output);
LavretStatus lavret_document_destroy(LavretCore *core, uint64_t document);
LavretStatus lavret_document_replace_source(LavretCore *core, uint64_t document, LavretDocumentSource source);
LavretStatus lavret_document_find_element_by_id(LavretCore *core, uint64_t document, LavretStringView id, uint64_t *output);
LavretStatus lavret_node_create_element(LavretCore *core, uint64_t document, LavretStringView name, uint64_t *output);
LavretStatus lavret_node_create_text(LavretCore *core, uint64_t document, LavretStringView text, uint64_t *output);
LavretStatus lavret_node_append(LavretCore *core, uint64_t document, uint64_t parent, uint64_t child);
LavretStatus lavret_node_insert_before(LavretCore *core, uint64_t document, uint64_t anchor, uint64_t node);
LavretStatus lavret_node_move(LavretCore *core, uint64_t document, uint64_t node, uint64_t new_parent, uint64_t before);
LavretStatus lavret_node_remove(LavretCore *core, uint64_t document, uint64_t node);
LavretStatus lavret_node_set_attribute(LavretCore *core, uint64_t document, uint64_t node, LavretStringView name, LavretStringView value);
LavretStatus lavret_node_remove_attribute(LavretCore *core, uint64_t document, uint64_t node, LavretStringView name);
LavretStatus lavret_node_replace_text(LavretCore *core, uint64_t document, uint64_t node, LavretStringView text);
LavretStatus lavret_document_set_viewport(LavretCore *core, uint64_t document, LavretViewport viewport);
LavretStatus lavret_document_resolve(LavretCore *core, uint64_t document, uint64_t animation_time_nanoseconds);
LavretStatus lavret_node_border_box(LavretCore *core, uint64_t document, uint64_t node, LavretRect *output);
LavretStatus lavret_document_hit_test(LavretCore *core, uint64_t document, LavretPoint point, LavretHandleArray *output);
LavretStatus lavret_resource_register(LavretCore *core, LavretResourceSource source, uint64_t *output);
LavretStatus lavret_resource_replace(LavretCore *core, uint64_t resource, LavretResourceSource source);
LavretStatus lavret_resource_remove(LavretCore *core, uint64_t resource);
LavretStatus lavret_render_target_create(LavretCore *core, uint32_t width, uint32_t height, uint64_t *output);
LavretStatus lavret_render_target_resize(LavretCore *core, uint64_t target, uint32_t width, uint32_t height);
LavretStatus lavret_render_target_destroy(LavretCore *core, uint64_t target);
LavretStatus lavret_render(LavretCore *core, uint64_t document, uint64_t target, LavretRenderResult *output);
LavretStatus lavret_render_target_readback_rgba8(LavretCore *core, uint64_t target, LavretReadback *output);
void lavret_readback_free(LavretReadback readback);
LavretStatus lavret_input_dispatch(LavretCore *core, uint64_t document, LavretInputEvent event);
LavretStatus lavret_ui_events_drain(LavretCore *core, LavretUiEventArray *output);
void lavret_ui_events_free(LavretUiEventArray events);
LavretStatus lavret_diagnostics_drain(LavretCore *core, LavretDiagnosticArray *output);
void lavret_diagnostics_free(LavretDiagnosticArray diagnostics);

#ifdef __cplusplus
}
#endif

#endif
