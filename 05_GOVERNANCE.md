# Governance Model

**The Constitution of Project V (Cottage Prime)**

---

## 1. The Council

The Council is the governing body for all architectural decisions. It consists of three roles:

### 1.1 The Architect (User)

- **Authority:** Final decision-maker with veto power
- **Responsibility:** Strategic direction, system design, priority setting
- **Mandate:** All architectural pivots require Architect approval

### 1.2 The Skeptic (Adversarial Reviewer)

- **Authority:** Challenge all assumptions
- **Responsibility:** Identify risks, edge cases, failure modes
- **Mantra:** "What could go wrong?"

### 1.3 The Quant (Mathematical Rigor)

- **Authority:** Validate mathematical correctness
- **Responsibility:** Ensure formulas, proofs, and statistical methods are sound
- **Mantra:** "Show me the math."

### 1.4 Council Review Process

All **architectural pivots** require a Council Review:

PROPOSAL → Architect or IDE drafts RFC

SKEPTIC → Adversarial review (risks, edge cases)

QUANT → Mathematical validation

DECISION → Architect approves/vetoes

RECORD → ADR created in docs/internal/adr/

> [!IMPORTANT]
> **No code is written until the Council has reviewed the proposal.**

---

## 2. Decision Making

### 2.1 Consensus Model

- **Preferred:** Unanimous agreement among Council roles
- **Fallback:** Majority with documented dissent
- **Override:** The Architect has absolute veto power

### 2.2 Decision Types

| Type | Review Required | Approver |
|------|-----------------|----------|
| **Trivial** (typos, formatting) | None | IDE |
| **Minor** (refactors, bug fixes) | Self-review | IDE |
| **Standard** (new features) | Council Review | Architect |
| **Major** (architecture changes) | Full Council + ADR | Architect |
| **Critical** (security, data) | Full Council + External | Architect |

### 2.3 Escalation Path

IDE → Skeptic Review → Quant Review → Architect Decision

---

## 3. Security Policy: The Black Box Doctrine

### 3.1 Information Classification

All project information is classified into two categories:

| Classification | Location | Contents |
|---------------|----------|----------|
| **PUBLIC** | `docs/` (this repo) | Usage docs, API specs, architecture overviews |
| **PRIVATE** | Confluence / Internal | Vital logic, mathematical proofs, strategy configs, ADRs |

### 3.2 The Black Box Rule

> [!CAUTION]
> **NEVER commit the following to this repository:**
>
> - Detailed alpha generation logic
> - Complete mathematical derivations that reveal edge
> - Strategy hyperparameters and tuning
> - API keys, credentials, or secrets
> - Internal Confluence links

### 3.3 What IS Public

- High-level architecture diagrams
- API endpoint schemas (not logic)
- Formula names (not derivations)
- Installation and usage instructions
- Component interfaces (not implementations)

### 3.4 What IS Private (Confluence)

- Complete mathematical proofs
- Strategy backtesting results
- Parameter optimization studies
- Trading performance data
- Incident post-mortems with P&L
- Architecture Decision Records (ADRs)

### 3.5 Review Checklist

Before every commit, verify:

- [ ] No secrets or credentials
- [ ] No detailed alpha logic
- [ ] No internal links
- [ ] No performance/P&L data
- [ ] README only contains "usage" level docs

---

## 4. Code of Conduct

### 4.1 Our Pledge

We pledge to make participation in this project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

---

*"The best code is the code that doesn't need to be written."* — The Council
