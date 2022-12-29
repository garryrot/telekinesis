#include "Config.h"

#include <articuno/archives/ryml/ryml.h>

using namespace articuno::ryml;

namespace Telekinesis {
    const Config& Config::GetSingleton() noexcept {
        static Config instance;

        static std::atomic_bool initialized;
        static std::latch latch(1);
        if (!initialized.exchange(true)) {
            std::ifstream inputFile(R"(Data\SKSE\Plugins\Telekinesis.yaml)");
            if (inputFile.good()) {
                yaml_source ar(inputFile);
                ar >> instance;
            }
            latch.count_down();
        }
        latch.wait();

        return instance;
    }
}