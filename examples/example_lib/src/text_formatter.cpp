#include <nlohmann/json.hpp>
#include <fmt/core.h>
#include <string>

int print_smt(std::string firstParam) {
    nlohmann::json json_obj;
    json_obj["input"] = firstParam;
    json_obj["length"] = firstParam.length();
    json_obj["is_empty"] = firstParam.empty();
    
    fmt::print("JSON output: {}\n", json_obj.dump(2));
    
    return 0;
}