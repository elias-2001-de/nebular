use crate::dom_api::DOMApi;
use crate::lua_dom::LuaDOM;
use mlua::{Lua, Result as LuaResult};
use std::sync::{Arc, Mutex};

pub struct LuaEngine {
    lua: Lua,
    dom: Arc<Mutex<LuaDOM>>,
}

impl LuaEngine {
    pub fn new(dom: LuaDOM) -> LuaResult<Self> {
        let lua = Lua::new();
        let dom = Arc::new(Mutex::new(dom));

        // Create a new engine instance
        let mut engine = LuaEngine {
            lua,
            dom: dom.clone(),
        };

        // Initialize the DOM API
        engine.setup_dom_api()?;

        Ok(engine)
    }

    fn setup_dom_api(&mut self) -> LuaResult<()> {
        let dom_clone = self.dom.clone();
        let dom_api = DOMApi::new(dom_clone);

        // Create the global 'dom' table
        let dom_table = self.lua.create_table()?;

        // Set up DOM manipulation functions
        {
            let dom_api_clone = dom_api.clone();
            dom_table.set("getElementById", self.lua.create_function(move |_, id: String| {
                dom_api_clone.get_element_by_id(&id)
            })?)?;
        }

        {
            let dom_api_clone = dom_api.clone();
            dom_table.set("addElement", self.lua.create_function(move |lua, element_table: mlua::Table| {
                dom_api_clone.add_element(lua, element_table)
            })?)?;
        }

        {
            let dom_api_clone = dom_api.clone();
            dom_table.set("removeElementById", self.lua.create_function(move |_, id: String| {
                dom_api_clone.remove_element_by_id(&id)
            })?)?;
        }

        {
            let dom_api_clone = dom_api.clone();
            dom_table.set("setElementText", self.lua.create_function(move |_, (id, text): (String, String)| {
                dom_api_clone.set_element_text(&id, &text)
            })?)?;
        }

        {
            let dom_api_clone = dom_api.clone();
            dom_table.set("setElementColor", self.lua.create_function(move |_, (id, color): (String, String)| {
                dom_api_clone.set_element_color(&id, &color)
            })?)?;
        }

        {
            let dom_api_clone = dom_api.clone();
            dom_table.set("addElementStyle", self.lua.create_function(move |_, (id, style): (String, String)| {
                dom_api_clone.add_element_style(&id, &style)
            })?)?;
        }

        {
            let dom_api_clone = dom_api.clone();
            dom_table.set("getAllElements", self.lua.create_function(move |lua, ()| {
                dom_api_clone.get_all_elements(lua)
            })?)?;
        }

        // Set the global dom variable
        self.lua.globals().set("dom", dom_table)?;

        Ok(())
    }

    pub fn execute_script(&self, script_content: &str) -> LuaResult<()> {
        self.lua.load(script_content).exec()
    }

    pub fn execute_script_file(&self, file_path: &str) -> LuaResult<()> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read script file {}: {}", file_path, e)))?;

        self.execute_script(&content)
    }

    pub fn get_dom(&self) -> Arc<Mutex<LuaDOM>> {
        self.dom.clone()
    }

    pub fn update_dom(&self, new_dom: LuaDOM) -> Result<(), Box<dyn std::error::Error>> {
        let mut dom_guard = self.dom.lock().map_err(|e| format!("Failed to lock DOM: {}", e))?;
        *dom_guard = new_dom;
        Ok(())
    }

    pub fn evaluate_expression(&self, expression: &str) -> LuaResult<mlua::Value> {
        self.lua.load(expression).eval()
    }

    pub fn call_function(&self, function_name: &str, args: Vec<mlua::Value>) -> LuaResult<mlua::Value> {
        let globals = self.lua.globals();
        let function: mlua::Function = globals.get(function_name)?;

        match args.len() {
            0 => function.call(()),
            1 => function.call(args[0].clone()),
            _ => function.call(mlua::MultiValue::from_vec(args)),
        }
    }

    pub fn set_global_variable(&self, name: &str, value: mlua::Value) -> LuaResult<()> {
        self.lua.globals().set(name, value)
    }

    pub fn get_global_variable(&self, name: &str) -> LuaResult<mlua::Value> {
        self.lua.globals().get(name)
    }
}