#!/bin/bash
# launch_agents.sh — no Docker, just terminals
# Usage: ./launch_agents.sh <num_agents>

set -e

NUM_AGENTS=${1:-2}
BARE_REPO="$(pwd)/upstream.git"
GITHUB_REMOTE="git@github.com:kiankyars/regex.git"

echo "=== Regex Agent Team Launcher ==="
echo "Agents: $NUM_AGENTS"

# --- Step 1: Create bare repo if needed ---
if [ ! -d "$BARE_REPO" ]; then
    echo "[setup] Creating bare repo..."
    git clone --bare "$GITHUB_REMOTE" "$BARE_REPO"
fi

# --- Step 2: Create workspace + launch for each agent ---
for i in $(seq 1 "$NUM_AGENTS"); do
    WORKSPACE="$(pwd)/workspace-${i}"

    if [ ! -d "$WORKSPACE" ]; then
        echo "[setup] Cloning workspace for agent $i..."
        git clone "$BARE_REPO" "$WORKSPACE"
    fi

    # Copy latest prompt and test harness into each workspace
    cp AGENT_PROMPT.md "$WORKSPACE/"
    cp test.sh "$WORKSPACE/"
    chmod +x "$WORKSPACE/test.sh"

    echo "[launch] Starting agent $i in background..."
    cd "$WORKSPACE"
    AGENT_ID=$i nohup bash ../agent_loop.sh > "../agent_${i}.out" 2>&1 &
    echo "[launch] Agent $i running (PID: $!)"
    cd ..
done

echo ""
echo "=== All $NUM_AGENTS agents running ==="
echo ""
echo "Useful commands:"
echo "  tail -f agent_1.out                # follow agent 1"
echo "  tail -f agent_2.out                # follow agent 2"
echo "  cd upstream.git && git log --oneline -20  # see commits"
echo "  kill \$(jobs -p)                     # stop all agents"
