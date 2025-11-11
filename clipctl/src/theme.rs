use ratatui::style::{Color, Modifier, Style};

/// Color theme for the TUI
#[derive(Debug, Clone)]
pub struct Theme {
    // UI elements
    pub border: Color,
    pub border_focused: Color,
    pub title: Color,
    pub background: Color,
    
    // List and selection
    pub list_item: Color,
    pub list_selected_bg: Color,
    pub list_selected_fg: Color,
    pub list_highlight_symbol: Color,
    
    // Content types
    pub text_icon: Color,
    pub url_icon: Color,
    pub image_icon: Color,
    pub rtf_icon: Color,
    pub code_icon: Color,
    
    // Metadata
    pub metadata_label: Color,
    pub metadata_value: Color,
    pub tag_fg: Color,
    pub tag_bg: Color,
    
    // Command bar
    pub command_prompt: Color,
    pub command_input: Color,
    
    // Help
    pub help_section: Color,
    pub help_key: Color,
    pub help_desc: Color,
}

impl Theme {
    /// Nord-inspired theme with cool blues and purples
    pub fn nord() -> Self {
        Self {
            border: Color::Rgb(129, 161, 193),           // Nord9 - light blue
            border_focused: Color::Rgb(136, 192, 208),   // Nord8 - bright cyan
            title: Color::Rgb(136, 192, 208),            // Nord8
            background: Color::Rgb(46, 52, 64),          // Nord0
            
            list_item: Color::Rgb(216, 222, 233),        // Nord4
            list_selected_bg: Color::Rgb(94, 129, 172),  // Nord10
            list_selected_fg: Color::Rgb(236, 239, 244), // Nord6
            list_highlight_symbol: Color::Rgb(163, 190, 140), // Nord14
            
            text_icon: Color::Rgb(136, 192, 208),        // Nord8 - cyan
            url_icon: Color::Rgb(129, 161, 193),         // Nord9 - blue
            image_icon: Color::Rgb(180, 142, 173),       // Nord15 - purple
            rtf_icon: Color::Rgb(235, 203, 139),         // Nord13 - yellow
            code_icon: Color::Rgb(163, 190, 140),        // Nord14 - green
            
            metadata_label: Color::Rgb(143, 188, 187),   // Nord7 - teal
            metadata_value: Color::Rgb(229, 233, 240),   // Nord5
            tag_fg: Color::Rgb(46, 52, 64),              // Nord0
            tag_bg: Color::Rgb(235, 203, 139),           // Nord13
            
            command_prompt: Color::Rgb(143, 188, 187),   // Nord7
            command_input: Color::Rgb(236, 239, 244),    // Nord6
            
            help_section: Color::Rgb(136, 192, 208),     // Nord8
            help_key: Color::Rgb(235, 203, 139),         // Nord13
            help_desc: Color::Rgb(216, 222, 233),        // Nord4
        }
    }
    
    /// Dracula theme with vibrant purples and pinks
    pub fn dracula() -> Self {
        Self {
            border: Color::Rgb(98, 114, 164),            // Dracula purple (dimmed)
            border_focused: Color::Rgb(189, 147, 249),   // Dracula purple
            title: Color::Rgb(189, 147, 249),            // Dracula purple
            background: Color::Rgb(40, 42, 54),          // Dracula background
            
            list_item: Color::Rgb(248, 248, 242),        // Dracula foreground
            list_selected_bg: Color::Rgb(68, 71, 90),    // Dracula current line
            list_selected_fg: Color::Rgb(255, 121, 198), // Dracula pink
            list_highlight_symbol: Color::Rgb(80, 250, 123), // Dracula green
            
            text_icon: Color::Rgb(139, 233, 253),        // Dracula cyan
            url_icon: Color::Rgb(189, 147, 249),         // Dracula purple
            image_icon: Color::Rgb(255, 121, 198),       // Dracula pink
            rtf_icon: Color::Rgb(241, 250, 140),         // Dracula yellow
            code_icon: Color::Rgb(80, 250, 123),         // Dracula green
            
            metadata_label: Color::Rgb(98, 114, 164),    // Dracula comment
            metadata_value: Color::Rgb(248, 248, 242),   // Dracula foreground
            tag_fg: Color::Rgb(40, 42, 54),              // Dracula background
            tag_bg: Color::Rgb(241, 250, 140),           // Dracula yellow
            
            command_prompt: Color::Rgb(80, 250, 123),    // Dracula green
            command_input: Color::Rgb(248, 248, 242),    // Dracula foreground
            
            help_section: Color::Rgb(189, 147, 249),     // Dracula purple
            help_key: Color::Rgb(255, 121, 198),         // Dracula pink
            help_desc: Color::Rgb(248, 248, 242),        // Dracula foreground
        }
    }
    
    /// Tokyo Night theme with deep blues and vibrant accents
    pub fn tokyo_night() -> Self {
        Self {
            border: Color::Rgb(65, 72, 104),             // Tokyo Night border
            border_focused: Color::Rgb(122, 162, 247),   // Tokyo Night blue
            title: Color::Rgb(122, 162, 247),            // Tokyo Night blue
            background: Color::Rgb(26, 27, 38),          // Tokyo Night background
            
            list_item: Color::Rgb(192, 202, 245),        // Tokyo Night foreground
            list_selected_bg: Color::Rgb(41, 46, 66),    // Tokyo Night selection
            list_selected_fg: Color::Rgb(187, 154, 247), // Tokyo Night purple
            list_highlight_symbol: Color::Rgb(158, 206, 106), // Tokyo Night green
            
            text_icon: Color::Rgb(125, 207, 255),        // Tokyo Night cyan
            url_icon: Color::Rgb(122, 162, 247),         // Tokyo Night blue
            image_icon: Color::Rgb(187, 154, 247),       // Tokyo Night purple
            rtf_icon: Color::Rgb(224, 175, 104),         // Tokyo Night yellow
            code_icon: Color::Rgb(158, 206, 106),        // Tokyo Night green
            
            metadata_label: Color::Rgb(86, 95, 137),     // Tokyo Night comment
            metadata_value: Color::Rgb(192, 202, 245),   // Tokyo Night foreground
            tag_fg: Color::Rgb(26, 27, 38),              // Tokyo Night background
            tag_bg: Color::Rgb(224, 175, 104),           // Tokyo Night yellow
            
            command_prompt: Color::Rgb(158, 206, 106),   // Tokyo Night green
            command_input: Color::Rgb(192, 202, 245),    // Tokyo Night foreground
            
            help_section: Color::Rgb(122, 162, 247),     // Tokyo Night blue
            help_key: Color::Rgb(255, 158, 100),         // Tokyo Night orange
            help_desc: Color::Rgb(192, 202, 245),        // Tokyo Night foreground
        }
    }
    
    /// Gruvbox theme with warm, earthy tones
    pub fn gruvbox() -> Self {
        Self {
            border: Color::Rgb(146, 131, 116),           // Gruvbox gray
            border_focused: Color::Rgb(254, 128, 25),    // Gruvbox orange
            title: Color::Rgb(254, 128, 25),             // Gruvbox orange
            background: Color::Rgb(40, 40, 40),          // Gruvbox dark0
            
            list_item: Color::Rgb(235, 219, 178),        // Gruvbox fg
            list_selected_bg: Color::Rgb(80, 73, 69),    // Gruvbox dark2
            list_selected_fg: Color::Rgb(251, 184, 108), // Gruvbox yellow
            list_highlight_symbol: Color::Rgb(184, 187, 38), // Gruvbox green
            
            text_icon: Color::Rgb(131, 165, 152),        // Gruvbox aqua
            url_icon: Color::Rgb(131, 165, 152),         // Gruvbox aqua
            image_icon: Color::Rgb(211, 134, 155),       // Gruvbox purple
            rtf_icon: Color::Rgb(251, 184, 108),         // Gruvbox yellow
            code_icon: Color::Rgb(184, 187, 38),         // Gruvbox green
            
            metadata_label: Color::Rgb(146, 131, 116),   // Gruvbox gray
            metadata_value: Color::Rgb(235, 219, 178),   // Gruvbox fg
            tag_fg: Color::Rgb(40, 40, 40),              // Gruvbox dark0
            tag_bg: Color::Rgb(251, 184, 108),           // Gruvbox yellow
            
            command_prompt: Color::Rgb(184, 187, 38),    // Gruvbox green
            command_input: Color::Rgb(235, 219, 178),    // Gruvbox fg
            
            help_section: Color::Rgb(254, 128, 25),      // Gruvbox orange
            help_key: Color::Rgb(251, 184, 108),         // Gruvbox yellow
            help_desc: Color::Rgb(235, 219, 178),        // Gruvbox fg
        }
    }
    
    pub fn style_border(&self) -> Style {
        Style::default().fg(self.border)
    }
    
    pub fn style_border_focused(&self) -> Style {
        Style::default().fg(self.border_focused).add_modifier(Modifier::BOLD)
    }
    
    pub fn style_title(&self) -> Style {
        Style::default().fg(self.title).add_modifier(Modifier::BOLD)
    }
    
    pub fn style_list_item(&self) -> Style {
        Style::default().fg(self.list_item)
    }
    
    pub fn style_list_selected(&self) -> Style {
        Style::default()
            .fg(self.list_selected_fg)
            .bg(self.list_selected_bg)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn style_tag(&self) -> Style {
        Style::default()
            .fg(self.tag_fg)
            .bg(self.tag_bg)
            .add_modifier(Modifier::BOLD)
    }
    
    pub fn style_metadata_label(&self) -> Style {
        Style::default().fg(self.metadata_label).add_modifier(Modifier::ITALIC)
    }
    
    pub fn style_metadata_value(&self) -> Style {
        Style::default().fg(self.metadata_value)
    }
    
    pub fn style_command_prompt(&self) -> Style {
        Style::default().fg(self.command_prompt).add_modifier(Modifier::BOLD)
    }
    
    pub fn style_command_input(&self) -> Style {
        Style::default().fg(self.command_input)
    }
    
    pub fn style_help_section(&self) -> Style {
        Style::default().fg(self.help_section).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    }
    
    pub fn style_help_key(&self) -> Style {
        Style::default().fg(self.help_key).add_modifier(Modifier::BOLD)
    }
    
    pub fn style_help_desc(&self) -> Style {
        Style::default().fg(self.help_desc)
    }
}

