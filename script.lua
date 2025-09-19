-- Example Lua script for DOM manipulation

-- Change the dynamic content
dom.setElementText("dynamic_content", "This text was set by a Lua script!")
dom.setElementColor("dynamic_content", "cyan")

-- Add a new element
dom.addElement({
  type = "text",
  text = "This element was added by Lua script",
  color = "magenta",
  style = {"italic"},
  id = "script_added"
})

-- Modify the header
dom.addElementStyle("header", "underlined")