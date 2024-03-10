#include <string_view>
#include "../rust/target/cxxbridge/rust/cxx.h"

using namespace RE;
using namespace RE::BSScript;

// TODO: Move to separate file

#pragma warning( disable : 5103 ) // apparently something "does not result in a valid preprocessing token"

/*
 Whats? going on here?
 =====================

 We auto-generate type declarations for callbacks with up to 2 arguments.
 If you need other function signatures you can declare them yourself.

 Explanation:
 ===========

 IVitualMachine::RegisterFunction has a callback argumen that is a variadic template.
 Rust itself cannot do, so Porting the generic implementation of the function already
 seems to be impossible. I also did not find any way to refernce single implementations
 RegisterFunction with a specific callback in the cxx bridge, due to type conversion issues.

 Maybe there is a better way to do this, but at one point I stopped spending time on
 this and just auto-generate adapters with macros.

 Am I out of touch? No, it's the children who are wrong
*/

#define FN_CB_TYPE(NAME, ARGS_N, RET, ...) using NativeFn_ ##NAME## _ ##ARGS_N = RET## (*)(StaticFunctionTag* ## __VA_ARGS__);

#ifdef GENERATE_CB_BODIES
    #define FN_CB(NAME, RET_N, ARGS_N, ...) FN_CB_TYPE(NAME, ARGS_N, RET_N, __VA_ARGS__) \
        void RegisterFunc0(IVirtualMachine *vm, rust::Str name, rust::Str className, NativeFn_##NAME##_##ARGS_N callback) { \
            vm->RegisterFunction( (std::string) name, (std::string) className, callback ); \
        }
#else
    #define FN_CB(NAME, RET_N, ARGS_N, ...) FN_CB_TYPE(NAME, ARGS_N, RET_N, ## __VA_ARGS__) \
        void RegisterFunc0(IVirtualMachine *vm, rust::Str name, rust::Str className, NativeFn_##NAME##_##ARGS_N callback);
#endif

#define DEF_FNS_0(ARGS_N, ...) FN_CB(void, void, ARGS_N, ## __VA_ARGS__) \
                               FN_CB(bool, bool, ARGS_N, ## __VA_ARGS__) \
                               FN_CB(int, int, ARGS_N, ## __VA_ARGS__) \
                               FN_CB(float, float, ARGS_N, ## __VA_ARGS__) \
                               FN_CB(string, std::string, ARGS_N, ## __VA_ARGS__)
DEF_FNS_0(0) // +5 fns

#define DEF_FNS_1(ARGS_N, ...) DEF_FNS_0( ARGS_N## _int,, int, ## __VA_ARGS__) \
                               DEF_FNS_0( ARGS_N## _float,, float, ## __VA_ARGS__) \
                               DEF_FNS_0( ARGS_N## _nool,, bool, ## __VA_ARGS__) \
                               DEF_FNS_0( ARGS_N## _string,, std::string, ## __VA_ARGS__)
DEF_FNS_1(1) // +25 fns

#define DEF_FNS_2(ARGS_N, ...) DEF_FNS_1( ARGS_N## _int, int, ## __VA_ARGS__) \
                               DEF_FNS_1( ARGS_N## _float, float, ## __VA_ARGS__) \
                               DEF_FNS_1( ARGS_N## _bool, bool, ## __VA_ARGS__) \
                               DEF_FNS_1( ARGS_N## _string, std::string, ## __VA_ARGS__)
DEF_FNS_2(2) // +125 fns

// go beyond this at your own risk
