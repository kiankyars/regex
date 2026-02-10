#!/bin/bash
# agent_loop.sh — runs inside each Docker container

set -e

UPSTREAM="/upstream"
WORKSPACE="/workspace"

# Clone from the bare upstream repo
if [ ! -d "$WORKSPACE/.git" ]; then
    git clone "$UPSTREAM" "$WORKSPACE"
fi

cd "$WORKSPACE"
git config user.name "agent-${AGENT_ID:-0}"
git config user.email "agent-${AGENT_ID:-0}@regex-agents"

mkdir -p agent_logs

while true; do
    # Pull latest changes
    git pull --rebase origin main || git pull origin main

    COMMIT=$(git rev-parse --short=6 HEAD)
    LOGFILE="agent_logs/agent_${AGENT_ID:-0}_${COMMIT}_$(date +%s).log"

    echo "[$(date)] Agent ${AGENT_ID:-0} starting session at commit ${COMMIT}" | tee -a "$LOGFILE"

    claude --dangerously-skip-permissions \
           -p "$(cat AGENT_PROMPT.md)" \
           --model claude-opus-4-6 &>> "$LOGFILE"

    echo "[$(date)] Agent ${AGENT_ID:-0} session ended" >> "$LOGFILE"

    # Brief pause to avoid hammering if something goes wrong
    sleep 5
done
