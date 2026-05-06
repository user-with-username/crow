#include <fmt/core.h>
#include <string>

int print_smt(std::string firstParam) {
    fmt::print("Hello, {}! The answer is {}.\n", firstParam, 42);
    return 0;
}