local M = {}

---@class ESP32Opts
local defaults = {
  build_dir = "build.clang",
  baudrate = 115200,
  clangd_args = {},
}

M.options = vim.deepcopy(defaults)
M.state = {
  last_port = nil,
}

local function get_snacks()
  return require("snacks")
end

local function make_clangd_capabilities()
  local capabilities = vim.lsp.protocol.make_client_capabilities()

  -- clangd moved from the custom offsetEncoding extension to the standard
  -- positionEncodings capability in LSP 3.17.
  if capabilities.general then
    capabilities.general.positionEncodings = { "utf-16" }
  else
    capabilities.offsetEncoding = { "utf-16" }
  end

  return capabilities
end

function M.setup(opts)
  M.options = vim.tbl_deep_extend("force", M.options or {}, opts or {})
  M.ensure_clangd()
end

--- List available cu.* serial ports
function M.list_ports()
  local scandir = vim.uv.fs_scandir("/dev")
  local ports = {}
  local patterns = {
    "^cu%.",
    "^ttyUSB%d+$",
    "^ttyACM%d+$",
    "^tty%.usb",
    "^tty%.wchusbserial",
  }

  if scandir then
    while true do
      local name = vim.uv.fs_scandir_next(scandir)
      if not name then
        break
      end

      for _, pattern in ipairs(patterns) do
        if name:match(pattern) then
          table.insert(ports, { port = "/dev/" .. name })
          break
        end
      end
    end
  end

  table.sort(ports, function(a, b)
    return a.port < b.port
  end)

  return ports
end

--- Find the ESP-IDF specific clangd
function M.find_esp_clangd()
  -- if clangd is on the path, check if it is from espressif using.
  -- if so, we are done
  if vim.fn.executable("clangd") == 1 then
    -- use `clangd --version` to check if it is from espressif
    local clangd_version = vim.fn.system("clangd --version")
    if clangd_version:match("espressif") then
      -- return the absolute path to clangd
      return vim.fn.exepath("clangd")
    end
  end

  local home = vim.env.HOME or vim.fn.expand("~")
  local base = home .. "/.espressif/tools/esp-clang"
  local scandir = vim.uv.fs_scandir(base)
  if not scandir then
    return nil
  end

  local latest
  local latest_key
  while true do
    local name = vim.uv.fs_scandir_next(scandir)
    if not name then
      break
    end
    if name:match("^esp%-.+") then
      local candidate = base .. "/" .. name .. "/esp-clang/bin/clangd"
      if vim.fn.executable(candidate) == 1 and (not latest_key or name > latest_key) then
        latest = candidate
        latest_key = name
      end
    end
  end

  return latest
end

--- Ensure compile_commands.json exists
function M.ensure_compile_commands()
  local path = M.options.build_dir .. "/compile_commands.json"
  if vim.fn.filereadable(path) == 0 then
    vim.notify("[ESP32] ⚠️ Missing compile_commands.json in " .. path, vim.log.levels.WARN)
  end
end

--- Warn only when there is no clangd at all
function M.ensure_clangd()
  -- An esp-clang is better, because it knows the Xtensa target. Plain clangd
  -- still reads the project once espidf-index has built its database, so its
  -- absence is not worth a warning on every startup.
  if not M.find_esp_clangd() and vim.fn.executable("clangd") == 0 then
    vim.notify("[ESP32] No clangd found. C support will not work.", vim.log.levels.WARN)
  end
end

--- Open a Snacks terminal for idf.py command
function M.command(cmd, port)
  local Snacks = get_snacks()
  local opts = M.options
  local full_cmd = "idf.py -B " .. opts.build_dir
  local selected_port = port or M.state.last_port

  if selected_port then
    full_cmd = full_cmd .. " -p " .. selected_port
  end
  full_cmd = full_cmd .. " " .. cmd

  local terminal_opts = {
    win = {
      width = 0.6,
      height = 0.7,
      border = "single",
      title = "Ctrl + ] to stop",
      title_pos = "center",
    },
  }

  if cmd == "monitor" then
    Snacks.terminal.toggle(full_cmd, terminal_opts)
  else
    Snacks.terminal.open(full_cmd, terminal_opts)
  end
end

--- Run idf.py build
function M.build()
  local build_dir = M.options.build_dir
  M.command("build")
end

-- Setup build command for Neovim
vim.api.nvim_create_user_command("ESPBuild", function()
  M.build()
end, {})

--- Create Snacks picker for port and run idf.py command
function M.pick(cmd)
  local Snacks = get_snacks()
  Snacks.picker.pick({
    prompt = "Select ESP32 port",
    ui_select = true,
    focus = "list",
    finder = M.list_ports,
    layout = {
      layout = {
        box = "horizontal",
        width = 0.2,
        height = 0.2,
        {
          box = "vertical",
          border = "single",
          title = "USB Serial Ports",
          { win = "input", height = 1, border = "bottom" },
          { win = "list", border = "none" },
        },
      },
    },
    format = function(item)
      local a = Snacks.picker.util.align
      return { { a(item.port, 20, { align = "left" }) } }
    end,
    confirm = function(picker, item)
      picker:close()
      M.state.last_port = item.port
      M.command(cmd, item.port)
    end,
  })
end

--- Run idf.py reconfigure for build.clang
function M.reconfigure()
  local Snacks = get_snacks()
  local build_dir = M.options.build_dir
  Snacks.terminal.open("idf.py -B " .. build_dir .. " -D IDF_TOOLCHAIN=clang reconfigure", {
    win = {
      width = 0.5,
      height = 0.4,
      title = "ESP-IDF Reconfigure",
      title_pos = "center",
    },
  })
end

--- Set up ESP32 LSP configuration
function M.lsp_config()
  local clangd = M.find_esp_clangd() or "clangd"
  -- No --compile-commands-dir: for a project built on another Host the database
  -- is not in the tree, and ~/.config/clangd/config.yaml points at the copy
  -- espidf-index made. Passing a directory here would override that.
  local cmd = {
    clangd,
    "--background-index",
    "--clang-tidy",
    "--header-insertion=iwyu",
    "--completion-style=detailed",
    "--function-arg-placeholders",
    "--fallback-style=llvm",
  }

  vim.list_extend(cmd, M.options.clangd_args or {})

  return {
    cmd = cmd,
    -- Prefer the ESP-IDF project root and avoid falling back to a parent git
    -- repository, which breaks nested projects/monorepos.
    root_markers = { "sdkconfig", "CMakeLists.txt" },
    capabilities = make_clangd_capabilities(),
    init_options = {
      usePlaceholders = true,
      completeUnimported = true,
      clangdFileStatus = true,
    },
  }
end

--- Show ESP32 setup info
function M.info()
  local messages = {}

  -- clangd check
  if M.find_esp_clangd() then
    table.insert(messages, "✓ Found esp-clangd")
  else
    table.insert(messages, "✗ ESP-specific clangd missing")
  end

  -- Where clangd will actually read this project's flags from. A database in
  -- the tree is a local build; one in the cache was put there by espidf-index.
  local database = vim.fs.find("compile_commands.json", {
    path = vim.fn.getcwd(),
    upward = true,
    stop = vim.env.HOME,
  })[1]
  if database then
    table.insert(messages, "✓ compile database in the tree: " .. database)
  else
    local registered = vim.fn.expand("~/.config/clangd/config.yaml")
    local slug = vim.fn.fnamemodify(vim.fn.getcwd(), ":h:t") .. "-" .. vim.fn.fnamemodify(vim.fn.getcwd(), ":t")
    local cached = vim.fn.expand("~/.cache/espidf-index/projects/" .. slug .. "/compile_commands.json")
    if vim.fn.filereadable(cached) == 1 then
      table.insert(messages, "✓ compile database from espidf-index: " .. cached)
      table.insert(messages, "  built " .. vim.fn.strftime("%Y-%m-%d %H:%M", vim.fn.getftime(cached)))
    else
      table.insert(messages, "✗ no compile database; run: espidf-index " .. vim.fn.getcwd())
    end
    if vim.fn.filereadable(registered) == 0 then
      table.insert(messages, "✗ no " .. registered)
    end
  end

  -- idf.py and IDF_PATH matter only to the build and flash keymaps, which need
  -- a local ESP-IDF. They are unrelated to whether clangd can read the code.
  local function check_bin(bin)
    return vim.fn.executable(bin) == 1 and "✓" or "–"
  end

  table.insert(messages, check_bin("idf.py") .. " idf.py (local build only)")
  table.insert(messages, "IDF_PATH: " .. (vim.env.IDF_PATH or "– not set (remote build)"))

  vim.notify(table.concat(messages, "\n"), vim.log.levels.INFO)
end

-- Setup custom commands for Neovim
vim.api.nvim_create_user_command("ESPReconfigure", function()
  M.reconfigure()
end, {})

vim.api.nvim_create_user_command("ESPInfo", function()
  M.info()
end, {})

return M
