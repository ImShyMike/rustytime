#!/usr/bin/env bash
set -euo pipefail

COLS=$(tput cols)
CONFIG_PATH="$HOME/.wakatime.cfg"

# Display the banner
if (( COLS < 48 )); then
    echo -e "\e[1mWelcome to rustytime!\e[0m"
else
    echo -e "\e[36m"
    cat <<'EOF'
                 _         _   _                
  _ __ _   _ ___| |_ _   _| |_(_)_ __ ___   ___ 
 | '__| | | / __| __| | | | __| | '_ ` _ \ / _ \
 | |  | |_| \__ \ |_| |_| | |_| | | | | | |  __/
 |_|   \__,_|___/\__|\__, |\__|_|_| |_| |_|\___|
                     |___/                      
EOF
    echo -e "\e[0m"
fi

echo

# Backup existing config if it exists
if [[ -f "$CONFIG_PATH" ]]; then
  BACKUP_PATH="${CONFIG_PATH}.$(date +%s).bak"
  mv "$CONFIG_PATH" "$BACKUP_PATH"
  echo -e "\e[33m! Existing config file found. Backed up to $BACKUP_PATH\e[0m"
fi

# Extension/Plugin identifiers
VSCODE_EXT="WakaTime.vscode-wakatime"
JETBRAINS_PID="com.wakatime.intellij.plugin"

# Get config from env vars
API_KEY="${RT_API_KEY:-}"
API_URL="${RT_API_URL:-}"

# Exit if required env vars are not set
if [[ -z "$API_KEY" || -z "$API_URL" ]]; then
  echo -e "\e[31mError: RT_API_KEY and RT_API_URL must be set\e[0m"
  exit 1
fi

# Create ~/.wakatime.cfg
cat > "$CONFIG_PATH" <<EOF
[settings]
api_key = $API_KEY
api_url = $API_URL
heartbeat_rate_limit_seconds = 60
EOF

echo -e "\e[32m✓ Configuration file created at $CONFIG_PATH\e[0m\n"

echo -e "\e[1mInstalling WakaTime extensions...\e[0m"

vscode_extension_install() {
    local ide="$1"
    if command -v "$ide" >/dev/null 2>&1; then
        echo -e "\n\e[32m→ Installing WakaTime for $ide...\e[0m"
        "$ide" --install-extension "$VSCODE_EXT" --force
    else
        echo -e "\e[90m$ide CLI not found; skipping.\e[0m"
    fi
}

# VSCode
vscode_extension_install code

# Trae AI
vscode_extension_install trae

# Cursor
vscode_extension_install cursor

# Windsurf
vscode_extension_install windsurf

# JetBrains IDEs
JETBRAINS_BINARIES=(
  idea
  pycharm
  pycharm1
  pycharm2
  clion
  goland
  webstorm
  rider
  datagrip
  phpstorm
  rubymine
  appcode
  rustrover
)

found_jetbrains=false
for bin in "${JETBRAINS_BINARIES[@]}"; do
  if command -v "$bin" >/dev/null 2>&1; then
    found_jetbrains=true
    echo -e "\n\e[32m→ Installing WakaTime plugin in $bin...\e[0m"
    "$bin" installPlugins "$JETBRAINS_PID"
  fi
done

if ! $found_jetbrains; then
  echo -e "\e[90m\nNo JetBrains IDEs found; skipping.\e[0m"
fi

# Read saved config
echo -e "\n\e[1mReading the config file...\e[0m"
echo -e "\e[32m✓ Successfully read config:\e[0m"
echo -e "API URL:\e[90m $API_URL\e[0m"
echo -e "API Key:\e[90m ${API_KEY:0:8}...\e[0m"

# Send test heartbeat
echo -e "\n\e[1mSending test heartbeat...\e[0m"
response=$(curl -s -w "\n%{http_code}" -X POST "$API_URL/users/current/heartbeats" \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d "[{\"type\":\"file\",\"time\":$(date +%s),\"entity\":\"test.txt\",\"language\":\"Text\"}]")

http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | sed '$d')

# Check response
if [ "$http_code" = "200" ] || [ "$http_code" = "202" ]; then
  echo -e "\e[32m✓ Test heartbeat sent successfully!\e[0m"
  echo -e "\n\e[32m\e[1m✓ Installation complete!\e[0m"
  echo "Please restart your editors/IDEs for changes to take effect."
else
  echo -e "\e[31mError: Failed to send heartzbeat: $body"
  exit 1
fi 
