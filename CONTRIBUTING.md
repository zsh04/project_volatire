# Contributing Guide

**The Rules of Engagement for Project V**

---

## Quick Reference

| Section | What You Learn |
|---------|----------------|
| [Workflow](#workflow) | How to submit contributions |
| [Style Guide](#style-guide) | Writing, Python, and Rust standards |
| [Documentation](#documentation) | Diátaxis classification |
| [Security](#security) | What you can and cannot commit |

---

## Workflow

### 1. Check for Existing Work

Search GitHub Issues for existing Directives before starting.

### 2. Create a Directive

If no Directive exists, create one following the `PROJECT_MANAGEMENT.md` standard.

### 3. Wait for Approval

> **Do not write code until the Directive enters REVIEW or ACTIVE status.**

### 4. Create a Branch

```bash
git checkout -b dir-{id}/{short-description}


###  5. Make Changes
Follow the style guides below.

###6. Commit

```bash
git commit -m "feat: implement feature X [Dir-{ID}]"

### Style Guide

#### Code Style

##### Rust (The Reflex)

Tool Purpose
cargo fmt Formatter (Default)
cargo clippy Linter (Strict)
O(1) Mandate No dynamic allocation (Vec::push) in hot loops
Safe Rust No unsafe unless explicitly authorized by Council

```Rust
// Good: Pre-allocated buffer
let mut buffer: VecDeque<f64> = VecDeque::with_capacity(1000);

// Bad: Dynamic allocation in tick loop
buffer.push_back(price); // Potential reallocation
```

### Python (The Brain)

Tool Purpose
Black Formatter
Ruff Linter
mypy Strict Type Checking
Pydantic All data structures must be Models

```Python
class StrategyIntent(BaseModel):
    action: Literal['BUY', 'SELL', 'HOLD']
    conviction: float = Field(ge=0.0, le=1.0)
    reasoning: str
Commit Messages
```Plaintext
<type>: <description> [Dir-{ID}]

Type Description
featNew feature
fix Bug fix
docs Documentation only
refactor Code restructure (no behavior change)
test Tests
chore Build/tooling
perf Performance improvement (Must cite metrics)
### Documentation
Diátaxis Classification
All documentation changes must classify content into one of four quadrants:
Quadrant Purpose Location
Tutorial Learning-oriented (step-by-step)docs/public/tutorials/
How-To Task-oriented (solve a problem)docs/public/how-to/
Reference Information-oriented (technical specs)docs/public/reference/
Explanation Understanding-oriented (why decisions)docs/public/explanation/
Security The Black Box Doctrine
NEVER commit alpha logic or specific parameter weights to docs/public/.
Use docs/internal/ for sensitive derivations. This folder is git-ignored.
"Good documentation is better than good excuses."
