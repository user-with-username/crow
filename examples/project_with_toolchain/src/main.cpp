#include <iostream>
#include <fstream>
#include <vector>

struct Record {
    int number;
    float weight;
};

int main() {
    std::ifstream file("data.bin", std::ios::binary);
    std::vector<Record> records;
    
    while (file) {
        Record r;
        file.read(reinterpret_cast<char*>(&r), sizeof(Record));
        if (file) records.push_back(r);
    }
    
    Record max_r = {0, 0.0f};
    for (const auto& r : records) {
        if (r.weight > max_r.weight) max_r = r;
    }
    
    std::cout << max_r.number 
              << max_r.weight;
}