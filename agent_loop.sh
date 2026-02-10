#!/bin/bash
# agent_loop.sh — run in a screen session per agent
# Usage: AGENT_ID=1 ./agent_loop.sh

AGENT_ID="${AGENT_ID:-0}"
REPO="git@github.com:kiankyars/regex.git"
WORKSPACE="$(pwd)/workspace-${AGENT_ID}"

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
