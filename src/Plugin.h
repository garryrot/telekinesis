#include <string_view>
#include "../rust/target/cxxbridge/rust/cxx.h"

using namespace RE;
using namespace RE::BSScript;


// #################################################

using NativeFuncImpl_Void_0 = void(*)(StaticFunctionTag*);
void RegisterFunc0( IVirtualMachine *vm,
                    rust::Str name,
                    rust::Str className,
                    NativeFuncImpl_Void_0 callback);

// #################################################

