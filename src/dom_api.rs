use crate::lua_dom::{DOMElement, LuaDOM};
use mlua::{Lua, Result as LuaResult, Table};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DOMApi {
    dom: Arc<Mutex<LuaDOM>>,
}

impl DOMApi {
    pub fn new(dom: Arc<Mutex<LuaDOM>>) -> Self {
        DOMApi { dom }
    }

    pub fn get_element_by_id(&self, id: &str) -> LuaResult<Option<Table>> {
        let dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        let element = dom_guard.elements.iter().find(|elem| {
            elem.id.as_ref().map_or(false, |elem_id| elem_id == id)
        });

        if let Some(elem) = element {
            // Create a temporary Lua instance to create the table
            let lua = Lua::new();
            Ok(Some(elem.to_lua_table(&lua)?))
        } else {
            Ok(None)
        }
    }

    pub fn add_element(&self, _lua: &Lua, element_table: Table) -> LuaResult<()> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        let element = DOMElement::from_lua_table(&element_table)?;
        dom_guard.add_element(element);

        Ok(())
    }

    pub fn remove_element_by_id(&self, id: &str) -> LuaResult<bool> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        Ok(dom_guard.remove_element_by_id(id))
    }

    pub fn set_element_text(&self, id: &str, text: &str) -> LuaResult<bool> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        if let Some(element) = dom_guard.get_element_by_id(id) {
            element.text = text.to_string();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn set_element_color(&self, id: &str, color: &str) -> LuaResult<bool> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        if let Some(element) = dom_guard.get_element_by_id(id) {
            element.color = Some(color.to_string());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn add_element_style(&self, id: &str, style: &str) -> LuaResult<bool> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        if let Some(element) = dom_guard.get_element_by_id(id) {
            if element.style.is_none() {
                element.style = Some(Vec::new());
            }
            if let Some(ref mut styles) = element.style {
                if !styles.contains(&style.to_string()) {
                    styles.push(style.to_string());
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn remove_element_style(&self, id: &str, style: &str) -> LuaResult<bool> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        if let Some(element) = dom_guard.get_element_by_id(id) {
            if let Some(ref mut styles) = element.style {
                if let Some(pos) = styles.iter().position(|s| s == style) {
                    styles.remove(pos);
                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub fn get_all_elements(&self, lua: &Lua) -> LuaResult<Table> {
        let dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        let elements_table = lua.create_table()?;

        for (index, element) in dom_guard.elements.iter().enumerate() {
            elements_table.set(index + 1, element.to_lua_table(lua)?)?;
        }

        Ok(elements_table)
    }

    pub fn get_element_count(&self) -> LuaResult<usize> {
        let dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        Ok(dom_guard.elements.len())
    }

    pub fn clear_all_elements(&self) -> LuaResult<()> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        dom_guard.elements.clear();
        Ok(())
    }

    pub fn set_title(&self, title: &str) -> LuaResult<()> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        dom_guard.title = title.to_string();
        Ok(())
    }

    pub fn get_title(&self) -> LuaResult<String> {
        let dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        Ok(dom_guard.title.clone())
    }

    pub fn set_border(&self, border: bool) -> LuaResult<()> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        dom_guard.border = border;
        Ok(())
    }

    pub fn set_margin(&self, margin: u16) -> LuaResult<()> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        dom_guard.margin = margin;
        Ok(())
    }

    pub fn insert_element_at(&self, _lua: &Lua, index: usize, element_table: Table) -> LuaResult<()> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        let element = DOMElement::from_lua_table(&element_table)?;

        if index <= dom_guard.elements.len() {
            dom_guard.elements.insert(index, element);
            Ok(())
        } else {
            Err(mlua::Error::RuntimeError(format!("Index {} out of bounds", index)))
        }
    }

    pub fn move_element(&self, from_index: usize, to_index: usize) -> LuaResult<bool> {
        let mut dom_guard = self.dom.lock()
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to lock DOM: {}", e)))?;

        if from_index < dom_guard.elements.len() && to_index < dom_guard.elements.len() {
            let element = dom_guard.elements.remove(from_index);
            dom_guard.elements.insert(to_index, element);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}