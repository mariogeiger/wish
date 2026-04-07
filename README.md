# Wish

Distributes people in various slots maximizing the global satisfaction, taking into account quotas for each slot.

Uses the [Hungarian algorithm](https://en.wikipedia.org/wiki/Hungarian_algorithm) to minimize the total penalty.

Live at https://wish.geiger.ink

## Rebuild and restart

```bash
cargo build -p wish-server --release && trunk build --release --config client/Trunk.toml && systemctl --user restart wish
```
