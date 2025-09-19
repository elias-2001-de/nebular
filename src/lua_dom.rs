use mlua::{Lua, Result as LuaResult, Table};
use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct LuaDOM {
    pub title: String,
    pub border: bool,
    pub margin: u16,
    pub elements: Vec<DOMElement>,
}

#[derive(Debug, Clone)]
pub struct DOMElement {
    pub element_type: String,
    pub text: String,
    pub color: Option<String>,
    pub style: Option<Vec<String>>,
    pub id: Option<String>,
}

impl LuaDOM {
    pub fn from_lua_file(lua: &Lua, file_path: &str) -> LuaResult<Self> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read file {}: {}", file_path, e)))?;

        let table: Table = lua.load(&content).eval()?;
        Self::from_lua_table(&table)
    }

    pub fn from_lua_table(table: &Table) -> LuaResult<Self> {
        let title: String = table.get("title").unwrap_or_else(|_| "Nebular".to_string());
        let border: bool = table.get("border").unwrap_or(true);
        let margin: u16 = table.get("margin").unwrap_or(1);

        let elements_table: Option<Table> = table.get("elements").ok();
        let mut elements = Vec::new();

        if let Some(elem_table) = elements_table {
            for pair in elem_table.pairs::<i32, Table>() {
                let (_, element_table) = pair?;
                elements.push(DOMElement::from_lua_table(&element_table)?);
            }
        }

        Ok(LuaDOM {
            title,
            border,
            margin,
            elements,
        })
    }

    pub fn get_element_by_id(&mut self, id: &str) -> Option<&mut DOMElement> {
        self.elements.iter_mut().find(|elem| {
            elem.id.as_ref().map_or(false, |elem_id| elem_id == id)
        })
    }

    pub fn add_element(&mut self, element: DOMElement) {
        self.elements.push(element);
    }

    pub fn remove_element_by_id(&mut self, id: &str) -> bool {
        if let Some(pos) = self.elements.iter().position(|elem| {
            elem.id.as_ref().map_or(false, |elem_id| elem_id == id)
        }) {
            self.elements.remove(pos);
            true
        } else {
            false
        }
    }
}

impl DOMElement {
    pub fn from_lua_table(table: &Table) -> LuaResult<Self> {
        let element_type: String = table.get("type").unwrap_or_else(|_| "text".to_string());
        let text: String = table.get("text").unwrap_or_else(|_| "".to_string());
        let color: Option<String> = table.get("color").ok();

        let style_table: Option<Table> = table.get("style").ok();
        let style = if let Some(style_t) = style_table {
            let mut styles = Vec::new();
            for pair in style_t.pairs::<i32, String>() {
                let (_, style_str) = pair?;
                styles.push(style_str);
            }
            Some(styles)
        } else {
            None
        };

        let id: Option<String> = table.get("id").ok();

        Ok(DOMElement {
            element_type,
            text,
            color,
            style,
            id,
        })
    }

    pub fn to_lua_table(&self, lua: &Lua) -> LuaResult<Table> {
        let table = lua.create_table()?;
        table.set("type", self.element_type.clone())?;
        table.set("text", self.text.clone())?;

        if let Some(ref color) = self.color {
            table.set("color", color.clone())?;
        }

        if let Some(ref styles) = self.style {
            let style_table = lua.create_table()?;
            for (i, style) in styles.iter().enumerate() {
                style_table.set(i + 1, style.clone())?;
            }
            table.set("style", style_table)?;
        }

        if let Some(ref id) = self.id {
            table.set("id", id.clone())?;
        }

        Ok(table)
    }

    pub fn parse_color(&self) -> Color {
        match self.color.as_deref() {
            Some("red") => Color::Red,
            Some("green") => Color::Green,
            Some("blue") => Color::Blue,
            Some("yellow") => Color::Yellow,
            Some("cyan") => Color::Cyan,
            Some("magenta") => Color::Magenta,
            Some("gray") | Some("grey") => Color::Gray,
            Some("white") => Color::White,
            Some("black") => Color::Black,
            _ => Color::White,
        }
    }

    pub fn parse_style_modifiers(&self) -> Modifier {
        let mut modifier = Modifier::empty();
        if let Some(ref styles) = self.style {
            for style in styles {
                match style.as_str() {
                    "bold" => modifier |= Modifier::BOLD,
                    "italic" => modifier |= Modifier::ITALIC,
                    "underlined" => modifier |= Modifier::UNDERLINED,
                    "dim" => modifier |= Modifier::DIM,
                    "reversed" => modifier |= Modifier::REVERSED,
                    "strikethrough" => modifier |= Modifier::CROSSED_OUT,
                    _ => {}
                }
            }
        }
        modifier
    }

    pub fn get_style(&self) -> Style {
        Style::default()
            .fg(self.parse_color())
            .add_modifier(self.parse_style_modifiers())
    }
}