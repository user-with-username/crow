#ifndef TEXT_FORMATTER_H
#define TEXT_FORMATTER_H

#include <string>
#include <vector>

namespace TextFormatter {
    enum class Alignment {
        LEFT,
        CENTER,
        RIGHT
    };

    std::string align(const std::string& text, int width, Alignment alignment = Alignment::LEFT, char fillChar = ' ');

    std::string trim(const std::string& text);

    std::string ltrim(const std::string& text);

    std::string rtrim(const std::string& text);

    std::string toUpper(const std::string& text);

    std::string toLower(const std::string& text);

    std::string formatParagraph(const std::string& text, int width, Alignment alignment = Alignment::LEFT);

    std::vector<std::string> wrapText(const std::string& text, int width);
}

#endif // TEXT_FORMATTER_H