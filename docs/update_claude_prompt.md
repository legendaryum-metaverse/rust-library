# Prompt: Update CLAUDE.md

Use this prompt to update `CLAUDE.md` when the project evolves.

## Context

`CLAUDE.md` is a critical onboarding document for future AI coding sessions. Its purpose is to **quickly orient new sessions** to the codebase without becoming outdated or overwhelming.

**Key principle**: This document should contain **timeless knowledge**, not point-in-time implementation details.

## Update Guidelines

### When to Update

Update `CLAUDE.md` when:
- ✅ Core architecture changes (new major features, patterns)
- ✅ Critical development rules change (testing requirements, workflows)
- ✅ Project metadata changes (version bumps, repository structure)
- ✅ Common pain points are discovered (add to troubleshooting)
- ✅ File organization changes significantly

**Don't update** for:
- ❌ Individual feature additions (unless they change patterns)
- ❌ Bug fixes
- ❌ Minor version bumps
- ❌ Implementation details that belong in code comments

### Content Structure (DO NOT CHANGE)

The document follows this structure for a reason:

1. **What is This?** - 30-second overview + current version
2. **Quick Start** - Make commands, critical test requirements
3. **Architecture Overview** - Core concepts, event flow
4. **Critical Development Rules** - Testing, adding features, code patterns
5. **Key Files Reference** - Where to find things
6. **Common Tasks** - Step-by-step recipes
7. **Troubleshooting** - Known issues and solutions

**Target length**: 150-200 lines (± 20%)

### Writing Principles

**DO**:
- Use clear, scannable headers
- Provide concrete examples with code snippets
- Explain the "why" for critical rules (e.g., why `--test-threads=1`)
- Include actual commands that work
- Reference file names, not line numbers
- Keep it actionable ("Add to X", not "There is an X")

**DON'T**:
- Add version-specific "Recent Changes" sections
- Include mermaid diagrams (they become stale)
- Speculate on future features/roadmap
- Duplicate information from code comments
- Use excessive emojis (2-3 per section max)
- Reference specific line numbers (they change)

### Update Process

```bash
# 1. Check current state
git log --oneline -10
cat Cargo.toml | grep version
make test  # Verify test count/setup hasn't changed

# 2. Identify what changed
git diff main CLAUDE.md  # If comparing branches
git log --since="1 month ago" --oneline  # Recent work

# 3. Update CLAUDE.md sections as needed
# - Update version in "What is This?"
# - Add new patterns to "Code Patterns" if architecture changed
# - Update file reference table if new core files added
# - Add troubleshooting entries if common issues emerged
# - Update "Common Tasks" if workflows changed

# 4. Verify length
wc -l CLAUDE.md  # Should be 150-200 lines

# 5. Test readability
# Ask yourself: "Can a new session understand this in 2 minutes?"
```

## Examples

### ✅ Good Updates

**Scenario**: New retry strategy added
```markdown
## Critical Development Rules

### Code Patterns

**DON'T**:
- Hard-code retry delays (use backoff strategies)

**DO**:
- Use `fibonacci_backoff()` for progressive delays
- Use `exponential_backoff()` for rapid retries
```

**Scenario**: New test pattern discovered
```markdown
## Troubleshooting

**Tests hang after 2 minutes**: Increase timeout in test with
`#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`
```

**Scenario**: Project structure changed
```markdown
## Key Files Reference

| File                     | Purpose                          |
|--------------------------|----------------------------------|
| `legend-saga/src/events.rs` | Event definitions (moved from root) |
```

### ❌ Bad Updates

**Too specific** (belongs in code comments):
```markdown
## Implementation Details

The `AuditHandler` in `events_consume.rs:127` uses Arc<Mutex<bool>>
to track processing state. The mutex is acquired at line 134...
```

**Version-specific** (gets stale immediately):
```markdown
## Recent Changes in v0.0.42

- Fixed bug in connection pooling
- Updated dependencies
- Added new test case for edge case
```

**Speculative** (may never happen):
```markdown
## Future Plans

We plan to add Kafka support, migrate to gRPC, and implement
machine learning-based routing in Q3 2025...
```

## Template Sections

Use these templates when adding new content:

### New Common Task
```markdown
### [Task Name]
```bash
# Brief description of what this does
command --with-flags argument

# Or step-by-step for multi-step tasks
step_one
step_two
```

Why this is needed: [one sentence explanation]
```

### New Troubleshooting Entry
```markdown
**[Symptom]**: [Solution in imperative form]
```

### New Code Pattern
```markdown
**[Category]**:
- [Rule as DON'T/DO statement]
```

## Review Checklist

Before committing updates to `CLAUDE.md`:

- [ ] Version number is current
- [ ] No references to specific line numbers
- [ ] File paths are accurate
- [ ] Commands have been tested
- [ ] Length is 150-200 lines (± 20%)
- [ ] No "Recent Changes" or "Future Roadmap" sections
- [ ] New content follows existing patterns
- [ ] A new AI session could understand the project in 2 minutes
- [ ] No duplicate information from other docs

## Anti-Patterns to Avoid

❌ **The "Complete Reference"**: Don't try to document everything. Link to code/docs instead.

❌ **The "Historical Archive"**: Don't track what changed when. That's what git is for.

❌ **The "Future Vision"**: Don't document plans. Document what exists now.

❌ **The "Debug Log"**: Don't add every edge case. Only common, recurring issues.

❌ **The "API Documentation"**: Don't duplicate function signatures. Explain patterns.

## Success Criteria

A good `CLAUDE.md` update means:

1. ✅ A new AI session can start contributing in < 5 minutes
2. ✅ Critical rules are impossible to miss (testing requirements, etc.)
3. ✅ Common tasks have copy-paste-ready commands
4. ✅ The document will stay accurate for 3-6 months
5. ✅ Engineers can use it as a quick reference too

---

**Remember**: This document serves future AI sessions, not humans. Optimize for:
- **Fast scanning** (clear headers, short paragraphs)
- **Actionable content** (commands > descriptions)
- **Timeless knowledge** (patterns > implementations)
