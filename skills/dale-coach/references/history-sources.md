# History sources for Dale Coach

Coach reads the task history of the harness it runs in. Use that harness's
section and no other; where a live tool description differs, it wins. In every
harness a read returns the conversation only: user and agent messages, never
tool outputs.

## Codex

- Discover: `list_threads` at its largest documented safe page, currently
  `limit: 50`. It exposes 50 summaries and no older-page cursor; disclose that
  limit when reached. Coverage of 51–100 needs supplied IDs. It is discovery
  only and cannot gate a known ID or prove absence.
- Keep Codex tasks only. Exclude ChatGPT chats unless the user asks for them.
- Read: `read_thread` with `includeOutputs: false`. Follow older-turn cursors
  only to verify a material pattern.
- Identity: task ID and host ID.

## T3 Code

- Discover: `t3_thread_list`, newest first, with `allProjects: true` unless
  the user names a project, paging with `cursor` up to the cohort limit. Leave
  `includeSubagents` off and filter out threads that are still running,
  queued, starting, preparing, or waiting.
- Read: `t3_thread_read` with the default `messages` view, never `activity`,
  and a modest `maxCharsPerItem`. Go to older turns with `beforePosition` only
  to verify a material pattern.
- Read top-level threads only. Reading a child thread's final result marks it
  delivered, which changes source state.
- `t3_thread_search` is not exhaustive: use it to find a thread the user
  names, never to establish coverage.
- Identity: thread ID, plus the environment ID for a linked machine. Record
  each thread's provider and model; a T3 history mixes agents.

## Claude Code

Claude Code has no history tool. Its sessions are files, one per session:

```text
~/.claude/projects/<project folder>/<session id>.jsonl
```

`CLAUDE_CONFIG_DIR` relocates `~/.claude` when set. A retrospective request
authorizes reading these files and nothing else in that directory.

- Discover: list the `.jsonl` files newest first by modification time, across
  project folders unless the user names a project. Use the modification time
  as the update time. Skip the current session, and treat a file modified in
  the last ten minutes as an active task.
- Read: project each file to its conversation. Keep `user` and `assistant`
  entries, their text, and the names of tools called. Drop tool results,
  thinking, attachments, request records, and entries marked `isMeta` or
  `isSidechain`:

  ```sh
  jq -r 'select((.type == "user" or .type == "assistant") and (.isMeta != true) and (.isSidechain != true))
    | .timestamp as $t | .type as $r
    | (.message.content | if type == "string" then [{type: "text", text: .}] else . end)[]
    | if .type == "text" then "\($t) \($r): \(.text)"
      elif .type == "tool_use" then "\($t) \($r) tool: \(.name)"
      else empty end' "<session file>"
  ```

  Read the result in pages, newest part first. Without `jq`, page through the
  file and apply the same filter by eye; never load whole tool results.
- Title: the last `ai-title` entry's `aiTitle`, when present.
- Identity: session ID and project folder. There is no host ID.
