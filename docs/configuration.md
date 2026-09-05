# Configuration and platform support

Glance reads `~/.glance/config.json`. Invalid configuration is reported on normal startup; the nonblocking hook entry point still exits successfully. Unknown configuration keys survive hook preference updates.

```json
{
  "model": "sonnet",
  "refresh_seconds": 30,
  "no_model": false,
  "prompt": "Prefer concise, concrete descriptions.",
  "cache_retention_days": 90
}
```

- Model precedence: `--model`, `GLANCE_MODEL`, configuration file, built-in default.
- `--refresh-seconds` overrides the configuration interval between summary calls. Growth must also settle and herdr must not report active work. `r` bypasses the interval for a manual refresh.
- `no_model` or `--no-model` prevents model invocation, including the `summarize` command. The panel still reads local metadata and caches.
- `prompt` appends instructions to Glance's summary prompt.
- `cache_retention_days` is optional. When set, panel startup removes expired summary caches; configuration and todo files are preserved.
- `GLANCE_HOME` overrides Glance's state directory. `CLAUDE_CONFIG_DIR` overrides the Claude transcript/settings directory.

Inspect cleanup before deleting:

```sh
glance-panel cache-clean --older-than-days 90 --dry-run
glance-panel cache-clean --older-than-days 90
```

Long sessions are summarized in forward chunks. The processed cursor advances only over the turns included in each pass. Individual long messages are still clipped to keep each pass compact. The footer reports successful cached summary calls and the CLI's estimated USD cost when available; this is not a billing or subscription-limit meter, and failed calls are not counted.

## Windows

The application builds on Windows and the IPC transport supports named pipes. Set `HERDR_SOCKET_PATH` to the pipe endpoint supplied by herdr; there is no guessed default endpoint on Windows. Unix keeps its existing socket default. Direct `--session` mode works without herdr.

CI covers Windows, macOS, and Linux. Transport tests exercise fragmented frames and deadlines using local named pipes on Windows and sockets on Unix. Interactive herdr integration still requires live platform verification.
