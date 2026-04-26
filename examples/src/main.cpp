#include <text_formatter.h>
#include <iostream>

int main() {
    using namespace TextFormatter;

    std::string text = "  Hello, World!  ";
    std::string longText = "This is an example of a very long text that needs to be nicely formatted with word wrapping and center alignment.";

    std::cout << "=== Text Formatting Library Demo ===\n\n";

    std::cout << "1. Trim whitespace:\n";
    std::cout << "Original: '" << text << "'\n";
    std::cout << "After trim(): '" << trim(text) << "'\n\n";

    std::cout << "2. Change case:\n";
    std::cout << "toUpper(): " << toUpper(trim(text)) << "\n";
    std::cout << "toLower(): " << toLower(trim(text)) << "\n\n";

    std::cout << "3. Alignment (width 30):\n";
    std::string shortText = trim(text);
    std::cout << "LEFT:   '" << align(shortText, 30, Alignment::LEFT) << "'\n";
    std::cout << "CENTER: '" << align(shortText, 30, Alignment::CENTER) << "'\n";
    std::cout << "RIGHT:  '" << align(shortText, 30, Alignment::RIGHT) << "'\n\n";

    std::cout << "4. Paragraph formatting (width 40, center aligned):\n";
    std::cout << formatParagraph(longText, 40, Alignment::CENTER) << "\n\n";

    std::cout << "5. Alignment with custom fill char (width 30, fill '*'):\n";
    std::cout << align("Important", 30, Alignment::CENTER, '*') << "\n";

    return 0;
}