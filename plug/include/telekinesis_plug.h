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

bool tk_connect_and_scan();

bool tk_connect();

bool tk_scan_for_devices();

bool tk_vibrate_all(int speed);

bool tk_vibrate_all_for(int speed, float duration_sec);

bool tk_stop_all();

void tk_close();

int8_t *tk_try_get_next_event();

void tk_free_event(int8_t *event);

bool tk_init_logging_stdout(LogLevel level);

bool tk_init_logging(LogLevel level, const char *_path);

} // extern "C"
