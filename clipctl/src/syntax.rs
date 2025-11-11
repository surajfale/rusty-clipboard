use once_cell::sync::Lazy;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxSet, SyntaxReference};
use syntect::util::LinesWithEndings;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

/// Detects if content looks like code based on heuristics
pub fn detect_code_language(content: &str) -> Option<&'static str> {
    let content_lower = content.to_lowercase();
    let lines: Vec<&str> = content.lines().collect();
    
    // Check for common code patterns
    if content.contains("fn main()") || content.contains("pub fn ") || content.contains("impl ") {
        return Some("rust");
    }
    
    if content.contains("function ") || content.contains("const ") || content.contains("let ") 
        || content.contains("=>") && content.contains("{") {
        return Some("javascript");
    }
    
    if content.contains("def ") || content.contains("import ") || content.contains("class ")
        || content_lower.contains("if __name__ == \"__main__\"") {
        return Some("python");
    }
    
    if content.contains("package ") || content.contains("func ") || content.contains("type ")
        && content.contains("struct") {
        return Some("go");
    }
    
    if content.contains("#include") || content.contains("int main") {
        return Some("cpp");
    }
    
    if content.contains("public class ") || content.contains("public static void main") {
        return Some("java");
    }
    
    if content.contains("<?php") || content_lower.contains("<?php") {
        return Some("php");
    }
    
    if content.contains("SELECT ") || content_lower.contains("select ") 
        && (content_lower.contains(" from ") || content_lower.contains(" where ")) {
        return Some("sql");
    }
    
    if content.starts_with("#!/bin/bash") || content.starts_with("#!/bin/sh") {
        return Some("bash");
    }
    
    // Check if it has common code patterns (brackets, semicolons, etc.)
    let has_code_chars = content.chars().filter(|&c| c == '{' || c == '}' || c == ';').count() > 3;
    let avg_line_len = if !lines.is_empty() {
        content.len() / lines.len()
    } else {
        0
    };
    
    // If it has code-like structure but no specific language detected
    if has_code_chars && avg_line_len < 100 {
        return Some("plaintext");
    }
    
    None
}

/// Highlights code using syntect and converts to ratatui Text
pub fn highlight_code(content: &str, language: Option<&str>) -> Text<'static> {
    let syntax = if let Some(lang) = language {
        SYNTAX_SET.find_syntax_by_token(lang)
            .or_else(|| SYNTAX_SET.find_syntax_by_extension(lang))
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
    } else {
        // Try to detect from content
        SYNTAX_SET.find_syntax_by_first_line(content)
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text())
    };
    
    highlight_with_syntax(content, syntax)
}

fn highlight_with_syntax(content: &str, syntax: &SyntaxReference) -> Text<'static> {
    let theme = &THEME_SET.themes["base16-ocean.dark"];
    let mut highlighter = HighlightLines::new(syntax, theme);
    
    let mut lines = Vec::new();
    
    for line in LinesWithEndings::from(content).take(100) { // Limit to 100 lines for performance
        if let Ok(ranges) = highlighter.highlight_line(line, &SYNTAX_SET) {
            let mut spans = Vec::new();
            
            for (style, text) in ranges {
                let fg_color = syntect_to_ratatui_color(style.foreground);
                let mut ratatui_style = Style::default().fg(fg_color);
                
                if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
                    ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
                }
                if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
                    ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
                }
                if style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
                    ratatui_style = ratatui_style.add_modifier(Modifier::UNDERLINED);
                }
                
                spans.push(Span::styled(text.to_string(), ratatui_style));
            }
            
            lines.push(Line::from(spans));
        }
    }
    
    Text::from(lines)
}

fn syntect_to_ratatui_color(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

/// Renders markdown-like text with basic formatting
pub fn render_formatted_text(content: &str) -> Text<'static> {
    let mut lines = Vec::new();
    
    for line in content.lines().take(100) {
        let mut spans = Vec::new();
        let trimmed = line.trim();
        
        // Detect headers
        if trimmed.starts_with("# ") {
            spans.push(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Rgb(122, 162, 247))
                    .add_modifier(Modifier::BOLD),
            ));
        } else if trimmed.starts_with("## ") {
            spans.push(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Rgb(125, 207, 255))
                    .add_modifier(Modifier::BOLD),
            ));
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            // Bullet points
            spans.push(Span::styled(
                "• ".to_string(),
                Style::default().fg(Color::Rgb(158, 206, 106)),
            ));
            spans.push(Span::raw(trimmed[2..].to_string()));
        } else if trimmed.starts_with("```") {
            // Code block markers
            spans.push(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(Color::Rgb(146, 131, 116))
                    .add_modifier(Modifier::DIM),
            ));
        } else if line.starts_with("    ") || line.starts_with("\t") {
            // Indented code
            spans.push(Span::styled(
                line.to_string(),
                Style::default().fg(Color::Rgb(131, 165, 152)),
            ));
        } else {
            // Regular text with inline code detection
            spans.extend(parse_inline_formatting(line));
        }
        
        lines.push(Line::from(spans));
    }
    
    Text::from(lines)
}

fn parse_inline_formatting(line: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '`' {
            // Inline code
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            
            let mut code = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch == '`' {
                    chars.next();
                    break;
                }
                code.push(chars.next().unwrap());
            }
            
            spans.push(Span::styled(
                format!("`{}`", code),
                Style::default()
                    .fg(Color::Rgb(184, 187, 38))
                    .bg(Color::Rgb(60, 60, 60)),
            ));
        } else if ch == '*' && chars.peek() == Some(&'*') {
            // Bold
            chars.next();
            if !current.is_empty() {
                spans.push(Span::raw(current.clone()));
                current.clear();
            }
            
            let mut bold_text = String::new();
            let mut found_end = false;
            while let Some(ch) = chars.next() {
                if ch == '*' && chars.peek() == Some(&'*') {
                    chars.next();
                    found_end = true;
                    break;
                }
                bold_text.push(ch);
            }
            
            if found_end {
                spans.push(Span::styled(
                    bold_text,
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            } else {
                current.push_str("**");
                current.push_str(&bold_text);
            }
        } else {
            current.push(ch);
        }
    }
    
    if !current.is_empty() {
        spans.push(Span::raw(current));
    }
    
    if spans.is_empty() {
        spans.push(Span::raw(line.to_string()));
    }
    
    spans
}

