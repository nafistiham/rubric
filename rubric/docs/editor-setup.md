# Editor Setup — LSP Integration

Rubric ships a built-in LSP server. Start it with:

```sh
rubric lsp
```

The server speaks JSON-RPC over stdin/stdout. Configure your editor to launch it as a language server for Ruby files.

---

## VS Code

No extension required. Install the [generic LSP client](https://marketplace.visualstudio.com/items?itemName=somebar.generic-lsp) extension, then add to your workspace or user `settings.json`:

```json
{
  "genericLsp.servers": [
    {
      "name": "rubric",
      "command": ["rubric", "lsp"],
      "filetypes": ["ruby"]
    }
  ]
}
```

Or use the [Ruby LSP](https://marketplace.visualstudio.com/items?itemName=Shopify.ruby-lsp) extension's addon mechanism if it supports generic servers.

---

## Neovim (nvim-lspconfig)

```lua
-- In your Neovim config (e.g. ~/.config/nvim/lua/lsp.lua):
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.rubric then
  configs.rubric = {
    default_config = {
      cmd = { 'rubric', 'lsp' },
      filetypes = { 'ruby' },
      root_dir = lspconfig.util.root_pattern('rubric.toml', '.rubocop.yml', 'Gemfile', '.git'),
      single_file_support = true,
    },
  }
end

lspconfig.rubric.setup {}
```

---

## Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "ruby"
language-servers = ["rubric"]

[language-server.rubric]
command = "rubric"
args = ["lsp"]
```

---

## Emacs (lsp-mode)

```elisp
;; In your Emacs config:
(with-eval-after-load 'lsp-mode
  (add-to-list 'lsp-language-id-configuration '(ruby-mode . "ruby"))
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection '("rubric" "lsp"))
    :activation-fn (lsp-activate-on "ruby")
    :server-id 'rubric)))
```

---

## Zed

In `~/.config/zed/settings.json`:

```json
{
  "lsp": {
    "rubric": {
      "binary": {
        "path": "rubric",
        "arguments": ["lsp"]
      }
    }
  },
  "languages": {
    "Ruby": {
      "language_servers": ["rubric"]
    }
  }
}
```

---

## What you get

- Violations appear as squiggles as you type (on each file save/change)
- `.rubric_todo.toml` suppressions are respected — suppressed violations don't appear
- `rubric.toml` is reloaded automatically when it changes
- Latency: < 100ms on files up to ~1000 lines
