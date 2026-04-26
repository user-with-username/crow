#include "text_formatter.h"
#include <sstream>
#include <cctype>

namespace TextFormatter {

    std::string ltrim(const std::string& text) {
        size_t start = text.find_first_not_of(" \t\n\r\f\v");
        return (start == std::string::npos) ? "" : text.substr(start);
    }

    std::string rtrim(const std::string& text) {
        size_t end = text.find_last_not_of(" \t\n\r\f\v");
        return (end == std::string::npos) ? "" : text.substr(0, end + 1);
    }

    std::string trim(const std::string& text) {
        return rtrim(ltrim(text));
    }

    std::string toUpper(const std::string& text) {
        std::string result = text;
        for (char& c : result) {
            c = std::toupper(static_cast<unsigned char>(c));
        }
        return result;
    }

    std::string toLower(const std::string& text) {
        std::string result = text;
        for (char& c : result) {
            c = std::tolower(static_cast<unsigned char>(c));
        }
        return result;
    }

    std::string align(const std::string& text, int width, Alignment alignment, char fillChar) {
        if (width <= static_cast<int>(text.length())) {
            return text;
        }

        int padding = width - text.length();
        std::string result;

        switch (alignment) {
            case Alignment::LEFT:
                result = text + std::string(padding, fillChar);
                break;
            case Alignment::RIGHT:
                result = std::string(padding, fillChar) + text;
                break;
            case Alignment::CENTER:
                int leftPadding = padding / 2;
                int rightPadding = padding - leftPadding;
                result = std::string(leftPadding, fillChar) + text + std::string(rightPadding, fillChar);
                break;
        }
        return result;
    }

    std::vector<std::string> wrapText(const std::string& text, int width) {
        std::vector<std::string> lines;
        std::istringstream wordsStream(text);
        std::string word;
        std::string currentLine;

        while (wordsStream >> word) {
            if (currentLine.empty()) {
                currentLine = word;
            } else if (static_cast<int>(currentLine.length() + word.length() + 1) <= width) {
                currentLine += " " + word;
            } else {
                lines.push_back(currentLine);
                currentLine = word;
            }
        }
        
        if (!currentLine.empty()) {
            lines.push_back(currentLine);
        }
        
        return lines;
    }

    std::string formatParagraph(const std::string& text, int width, Alignment alignment) {
        std::vector<std::string> wrapped = wrapText(text, width);
        std::string result;
        
        for (size_t i = 0; i < wrapped.size(); ++i) {
            result += align(wrapped[i], width, alignment);
            if (i != wrapped.size() - 1) {
                result += "\n";
            }
        }
        
        return result;
    }

}