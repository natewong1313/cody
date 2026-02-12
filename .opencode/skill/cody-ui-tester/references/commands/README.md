# Commands Reference

All available commands for the Cody UI Test Harness.

## Command Format

Commands are sent as JSON objects:

```json
{
  "id": "unique-id",
  "command": {
    "command_name": { "param": "value" }
  }
}
```

Responses:

```json
{
  "id": "unique-id",
  "status": "ok|error|timeout",
  "screenshot": "/path/to/screenshot.png",
  "error": "error message if any",
  "elements": [ ... ]
}
```

---

## click

Click on a UI element by its label or text content.

**Parameters:**
- `target` (string): The label/text of the element to click

**Example:**
```json
{"id":"1","command":{"click":{"target":"New Session"}}}
```

**Response:**
```json
{"id":"1","status":"ok","screenshot":null,"error":null,"elements":null}
```

**Notes:**
- The target string is matched against element labels
- If multiple elements match, the first one is clicked
- Fails with error if no element is found
- Automatically runs the UI after click

**Timeout:** 10 seconds

---

## type

Type text into the currently focused input field.

**Parameters:**
- `text` (string): The text to type

**Example:**
```json
{"id":"2","command":{"type":{"text":"Hello, World!"}}}
```

**Response:**
```json
{"id":"2","status":"ok","screenshot":null,"error":null,"elements":null}
```

**Notes:**
- Text is typed character by character
- Only alphanumeric characters and spaces are supported
- Special characters may not work correctly
- Make sure an input field is focused first (click on it)

**Timeout:** 10 seconds

---

## screenshot

Capture a screenshot of the current UI state.

**Parameters:**
- `name` (string): Base filename for the screenshot (without extension)

**Example:**
```json
{"id":"3","command":{"screenshot":{"name":"main_screen"}}}
```

**Response:**
```json
{"id":"3","status":"ok","screenshot":"/tmp/cody-screenshots/main_screen.png","error":null,"elements":null}
```

**Notes:**
- Screenshots are saved to `/tmp/cody-screenshots/`
- Format is PNG
- If a file with the same name exists, it may be overwritten
- The `screenshot` field in the response contains the full path

**Timeout:** 10 seconds

---

## get_state

Query the current UI state and get information about all visible elements.

**Parameters:** None

**Example:**
```json
{"id":"4","command":"get_state"}
```

**Response:**
```json
{
  "id": "4",
  "status": "ok",
  "screenshot": null,
  "error": null,
  "elements": [
    {
      "label": "New Session",
      "element_type": "Button",
      "rect": [10.0, 20.0, 100.0, 30.0]
    },
    {
      "label": "Session name:",
      "element_type": "Label",
      "rect": [10.0, 60.0, 80.0, 20.0]
    }
  ]
}
```

**Element Fields:**
- `label`: The text/label of the element (may be null)
- `element_type`: The type of element (Button, Label, TextInput, etc.)
- `rect`: [x, y, width, height] in screen coordinates

**Notes:**
- Elements are returned in tree order (top-to-bottom, left-to-right)
- Invisible elements may not be included
- Useful for finding element positions before clicking

**Timeout:** 10 seconds

---

## wait

Pause execution for a specified duration.

**Parameters:**
- `ms` (number): Duration to wait in milliseconds

**Example:**
```json
{"id":"5","command":{"wait":{"ms":1000}}}
```

**Response:**
```json
{"id":"5","status":"ok","screenshot":null,"error":null,"elements":null}
```

**Notes:**
- Useful for waiting for UI animations or async operations
- Minimum wait time is 1 second (enforced)
- Does not take a screenshot automatically

**Timeout:** Specified wait time (max 1 second enforced)

---

## key_press

Press a specific key.

**Parameters:**
- `key` (string): Name of the key to press

**Example:**
```json
{"id":"6","command":{"key_press":{"key":"Enter"}}}
```

**Supported Keys:**
- Special keys: `Enter`, `Return`, `Escape`, `Esc`, `Tab`, `Backspace`, `Space`
- Letters: `a`-`z` (case insensitive)
- Numbers: `0`-`9`

**Response:**
```json
{"id":"6","status":"ok","screenshot":null,"error":null,"elements":null}
```

**Notes:**
- Keys are pressed and released immediately
- Modifier keys (Shift, Ctrl, Alt) are not currently supported
- Useful for submitting forms (Enter) or navigating (Tab)

**Timeout:** 10 seconds

---

## Error Responses

When a command fails, you'll receive an error response:

```json
{
  "id": "1",
  "status": "error",
  "screenshot": null,
  "error": "Element not found: 'Unknown Button'",
  "elements": null
}
```

Common error messages:
- `"Element not found: 'X'"` - The target element doesn't exist
- `"Invalid JSON: ..."` - The command JSON is malformed
- `"Failed to read input: ..."` - Communication error
- `"Unknown key: 'X'"` - Invalid key name for key_press
- `"Cannot convert character to key: 'X'"` - Character not supported by type command

---

## Combining Commands

Commands are executed sequentially. To perform a workflow:

```json
{"id":"1","command":"get_state"}
{"id":"2","command":{"click":{"target":"New Session"}}}
{"id":"3","command":{"wait":{"ms":500}}}
{"id":"4","command":{"screenshot":{"name":"after_click"}}}
```

Each command runs independently and returns its own response.
