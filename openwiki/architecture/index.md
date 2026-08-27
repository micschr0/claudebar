# Files

- [Input parsing: forgiving Claude Code session JSON](input-parsing.md) - How claudebar's stdin session JSON is parsed into InputData via a forgiving Coerce deserializer so any malformed field degrades to None rather than aborting the render.
- [Architecture overview](overview.md)
- [Render pipeline: session JSON to ANSI status line](render-pipeline.md) - The single render hot path shared by the statusline hook and the TUI preview — parse stdin JSON into InputData, resolve Config, resolve theme/style, then compose segments into one ANSI status line via render_line.
