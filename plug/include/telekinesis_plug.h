#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

enum class LogLevel {
  Trace,
  Debug,
  Info,
  Warn,
  Error,
};

extern "C" {

void *tk_connect();

bool tk_scan_for_devices(const void *_tk);

bool tk_vibrate_all(const void *_tk, float speed);

bool tk_vibrate_all_for(const void *_tk, float speed, float duration_sec);

int8_t *tk_try_get_next_event(const void *_tk);

void tk_free_event(const void*, int8_t *event);

bool tk_stop_all(const void *_tk);

void tk_close(void *_tk);

bool tk_init_logging(LogLevel level, const char *_path);

} // extern "C"
