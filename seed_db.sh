#!/bin/bash

if [ -z "$API_KEY" ]; then
  echo "ERROR: API_KEY environment variable is not set."
  exit 1
fi

read -p "Host (default='http://localhost:3000'): " HOST
HOST=HOST | "http://localhost:3000"
USER_AGENT="wakatime/v1.115.2 (linux-6.14.1) go1.24.2 vscode/1.100.0 vscode-wakatime/25.0.3"

PROJECTS=("Alpha" "Beta" "Gamma" "Delta")
LANGUAGES=("python" "javascript" "go" "rust" "c++")
FILE_EXT=(".py" ".js" ".go" ".rs" ".cpp")
BRANCHES=("main" "dev" "feature/x" "bugfix/y")

generate_random_heartbeat() {
    project=${PROJECTS[$RANDOM % ${#PROJECTS[@]}]}
    language=${LANGUAGES[$RANDOM % ${#LANGUAGES[@]}]}
    ext=${FILE_EXT[$RANDOM % ${#FILE_EXT[@]}]}
    branch=${BRANCHES[$RANDOM % ${#BRANCHES[@]}]}
    name=$(tr -dc 'a-z' < /dev/urandom | head -c 8)
    entity="/home/user/Documents/GitHub/${project,,}/src/${name}${ext}"
    
    now=$(date +%s)
    offset=$(( (RANDOM << 15 | RANDOM) % 604800 ))
    base=$((now - offset))
    millis=$((RANDOM % 1000))
    timestamp=$(printf "%d.%03d" "$base" "$millis")

    is_write=$( [ $((RANDOM % 2)) -eq 1 ] && echo true || echo false )
    lineno=$((1 + RANDOM % 100))
    lines=$((1 + RANDOM % 1000))
    cursorpos=$((RANDOM % 500))
    
  cat <<EOF
{
  "entity": "$entity",
  "type": "file",
  "category": "coding",
  "time": $timestamp,
  "project": "$project",
  "language": "$language",
  "branch": "$branch",
  "is_write": $is_write,
  "cursorpos": $cursorpos,
  "lineno": $lineno,
  "lines": $lines,
  "user_agent": "$USER_AGENT"
}
EOF
}

for i in {1..1000}; do
    json=$(generate_random_heartbeat)
    response=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$HOST/api/v1/users/current/heartbeats" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $API_KEY" \
        -H "User-Agent: $USER_AGENT" \
    -d "$json")
    
    if [ "$response" = "201" ]; then
        echo "[$response] Sent heartbeat $i/1000"
    else
        echo "[$response] ERROR sending heartbeat $i/1000"
    fi
done
