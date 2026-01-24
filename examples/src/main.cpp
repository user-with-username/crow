#include <iostream>
#include "calculator.h"

int main() {
    double a = 10.5;
    double b = 2.5;
    std::cout << "Addition: " << add(a, b) << "\n";
    std::cout << "Subtraction: " << subtract(a, b) << "\n";
    std::cout << "Multiplication: " << multiply(a, b) << "\n";
    
    try {
        std::cout << "Division: " << divide(a, b) << "\n";
        std::cout << "Division by 0: " << divide(a, 0) << "\n";
    } catch (const std::runtime_error& e) {
        std::cout << "Error: " << e.what() << "\n";
    }
    
    return 0;
}