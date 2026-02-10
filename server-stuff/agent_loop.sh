#!/bin/bash
# agent_loop.sh — run in a screen session per agent
# Usage: AGENT_ID=1 ./agent_loop.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_ID="${AGENT_ID:-0}"

if [ -z "${REPO:-}" ]; then
    if [ -d "${SCRIPT_DIR}/../regex/.git" ]; then
        REPO="${SCRIPT_DIR}/../regex"
    elif [ -d "${SCRIPT_DIR}/../.git" ]; then
        REPO="${SCRIPT_DIR}/.."
    else
        REPO="git@github.com:kiankyars/regex.git"
    fi
fi

if [ -d "$REPO/.git" ]; then
    REPO_PATH="$(cd "$REPO" && pwd)"
    WORKSPACE_ROOT="$(cd "$(dirname "$REPO_PATH")" && pwd)"
else
    WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
fi

WORKSPACE="${WORKSPACE_ROOT}/workspace-${AGENT_ID}"
PROMPT_PATH="${SCRIPT_DIR}/AGENT_PROMPT.md"

if [ ! -f "$PROMPT_PATH" ]; then
    echo "ERROR: AGENT_PROMPT.md not found at ${PROMPT_PATH}" >&2
    exit 1
fi

# Clone if needed
if [ ! -d "$WORKSPACE/.git" ]; then
    git clone "$REPO" "$WORKSPACE"
fi

cd "$WORKSPACE"
git config user.name "agent-${AGENT_ID}"
git config user.email "agent-${AGENT_ID}@regex-agents"

mkdir -p agent_logs current_tasks notes

while true; do
    git pull --rebase origin main 2>/dev/null || git pull origin main

    COMMIT=$(git rev-parse --short=6 HEAD)
    LOGFILE="agent_logs/agent_${AGENT_ID}_${COMMIT}_$(date +%s).log"

    echo "[$(date)] Agent ${AGENT_ID} starting session at ${COMMIT}"

    claude --dangerously-skip-permissions \
           -p "$(cat AGENT_PROMPT.md)" \
           --model claude-opus-4-6 &>> "$LOGFILE"

    EXIT_CODE=$?
    echo "[$(date)] Agent ${AGENT_ID} session ended (exit $EXIT_CODE)"

    if [ $EXIT_CODE -ne 0 ]; then
        echo "[$(date)] Backing off 60s..."
        sleep 60
    else
        sleep 5
    fi
done
