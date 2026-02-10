# Regex Engine — Agent Team Project

A regex engine built from scratch in Rust by a team of parallel Claude agents, with no external dependencies.

Inspired by [Nicholas Carlini's C compiler agent team experiment](https://www.anthropic.com/engineering/building-c-compiler).

## Goal

Build a fully-featured regex engine autonomously using parallel Claude Code agents, while experimenting with improvements to inter-agent coordination.

## Architecture

```
upstream.git (bare repo) — shared state between agents
├── Container 1 (Agent 1) — clones, works, pushes
├── Container 2 (Agent 2) — clones, works, pushes
└── ...
```

Agents coordinate via:
- `current_tasks/` — lock files to prevent duplicate work
- `notes/` — inter-agent communication (observations, warnings, failed approaches)
- `PROGRESS.md` — shared progress tracker

## Running

```bash
export ANTHROPIC_API_KEY=your-key-here
./launch_agents.sh 2  # start 2 agents
```

## Testing

```bash
./test.sh          # full test suite
./test.sh --fast   # 10% deterministic sample
```

Tests use Python's `re` module as an oracle for correctness.

## Metrics Tracked

- Tokens consumed per useful commit
- Merge conflict rate
- Duplicate work rate (how often agents attempt the same task)
- Test pass rate over time
- Time per session
