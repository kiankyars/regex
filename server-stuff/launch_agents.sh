#!/bin/bash
# launch_agents.sh — spawns agents in screen sessions
# Usage: ./launch_agents.sh <num_agents>

NUM_AGENTS=${1:-2}

echo "=== Regex Agent Team Launcher ==="

for i in $(seq 1 "$NUM_AGENTS"); do
    echo "[launch] Starting agent $i in screen 'agent-$i'..."
    screen -dmS "agent-$i" bash -c "AGENT_ID=$i ./agent_loop.sh"
    echo "[launch] Agent $i running"

    # Stagger starts to avoid simultaneous API hits
    if [ $i -lt "$NUM_AGENTS" ]; then
        sleep 30
    fi
done

echo ""
echo "=== All $NUM_AGENTS agents running ==="
echo ""
echo "Useful commands:"
echo "  screen -r agent-1          # attach to agent 1"
echo "  screen -r agent-2          # attach to agent 2"
echo "  screen -ls                 # list sessions"
echo "  Ctrl+A then D              # detach from a screen"
echo "  screen -S agent-1 -X quit  # kill agent 1"
