(fenced_code_block
  (info_string
    (language) @injection.language)
  (code_fence_content) @injection.content)

((inline) @injection.content
 (#set! injection.language "markdown-inline"))

((html_block) @injection.content
  (#set! injection.language "html"))

((minus_metadata) @injection.content (#set! injection.language "yaml"))

((plus_metadata) @injection.content (#set! injection.language "toml"))


((inline) @injection.content
  (#match? @injection.content "^\s*\(import\|export\)")
  (#set! injection.language "typescript"))

((inline) @injection.content
    (#match? @injection.content "<[A-Za-z][a-zA-Z0-9]*[^>]*\\/?\\>")
    (#set! injection.language "html"))
