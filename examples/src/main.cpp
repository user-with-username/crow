#include <iostream>
#include "calculator.h"

int main() {
    double a = 10.5;
    double b = 2.5;
    std::cout << "Addition: " << add(a, b) << "\n";
    std::cout << "Subtraction: " << subtract(a, b) << "\n";
    std::cout << "Multiplication: " << multiply(a, b) << "\n";
    
    return 0;
}