# Project Management Policy

**The Directive-Driven Workflow**

---

## 1. The Unit of Work: Directives

> [!IMPORTANT] > **We utilize the Atlassian Suite (Jira & Confluence) integrated via MCP tools. However, we reject vague "tickets." We use Directives.**
A **Directive** is a structured task assignment that provides complete context for execution. Every piece of work—from bug fixes to architectural pivots—is framed as a Directive within Jira. If it is not a Directive, it does not exist.

---

## 2. Directive Structure

Every Directive follows this Markdown format:

```markdown
## Directive-{ID}: {Title}

**Identity:** {Role performing the work}
**Context:** @{files involved}
**Objective:** {Definition of Done}
**Status:** {DRAFT | REVIEW | ACTIVE | VERIFIED}

### Background
{Why this work is needed. Link to Architecture/Philosophy axioms.}

### Requirements
1. {Specific requirement 1}
2. {Specific requirement 2}
...

### Acceptance Criteria
- [ ] {Criterion 1}
- [ ] {Criterion 2}
...

### Notes
{Any additional context, constraints, or considerations}

### 2.1 Required Fields

**Field** **Description** **Example** 
**Identity** Who is doing the work "The Architect", "The IDE", "The Quant"
**Context** Files/systems involved `@src/reflex/risk.rs`, `@docs/01_PHILOSOPHY.md`
**Objective** Clear Definition of Done "Implement Hill Estimator with $\alpha < 2.0$ veto"
**Status** Current lifecycle stage `ACTIVE`

### 2.2 Identity Roles

**Identity** **Description**
**The Architect** Strategic decisions, approvals, and defining the "Why".
**The IDE** Code implementation, refactoring, and execution.
**The Skeptic** Adversarial review (identifying edge cases and ruin scenarios).
**The Quant** Mathematical validation (ensuring the Physics is sound).
**The Ops** Infrastructure, Docker, Deployment, and Telemetry.

----------

## 3. Directive Lifecycle
The lifecycle enforces the **"Measure Twice, Cut Once"** philosophy.

Code snippet

graph LR
    DRAFT -->|Submit| REVIEW
    REVIEW -->|Approve| ACTIVE
    REVIEW -->|Reject| DRAFT
    ACTIVE -->|Complete| VERIFIED

### 3.1 DRAFT

-   **Definition:** Initial ideation and scoping.   
-   **Owner:** Architect or IDE.   
-   **Activities:** Define requirements, identify context files, set acceptance criteria.  
-   **Exit Criteria:** Directive is complete, coherent, and aligned with `01_PHILOSOPHY.md`.
    
### 3.2 REVIEW

-   **Definition:** The Council evaluates for bugs, logic flaws, and ruin risks.   
-   **Owner:** Skeptic + Quant.    
-   **Activities:**    
    -   **Adversarial Review:** "What happens if the exchange disconnects here?"        
    -   **Math Validation:** "Is this $O(1)$? Is the formula correct?"        
-   **Exit Criteria:** Council approval or rejection with feedback.   

### 3.3 ACTIVE

-   **Definition:** Implementation in progress. **Code can now be written.**    
-   **Owner:** IDE.    
-   **Activities:** Coding, Testing, Updating Documentation.    
-   **Exit Criteria:** All acceptance criteria met, tests passed.    

### 3.4 VERIFIED

-   **Definition:** User has confirmed the output.    
-   **Owner:** Architect.    
-   **Activities:** Final review, Acceptance testing, Sign-off.    
-   **Mandatory:** The IDE must self-verify against `docs/internal/process/DEFINITION_OF_DONE.md` before requesting verification.   

----------

## 4. Traceability & Version Control
We maintain a strict link between **Directives** and **Code**.

### 4.1 Commit Message Format
Every commit must reference the Directive ID.

Example 
<type>: <description> [Dir-{ID}]
<optional body> 

**Types:**
-   `feat`: New feature    
-   `fix`: Bug fix    
-   `docs`: Documentation only    
-   `refactor`: Code change that neither fixes a bug nor adds a feature    
-   `test`: Adding or updating tests    
-   `chore`: Build process or auxiliary tool changes
    
Example:
feat: implement Maker-Only execution logic [Dir-22]

### 4.2 Branch Naming
Branches are ephemeral workspaces for Directives.
Format: `dir-{id}/{short-description}`
Example:
dir-22/maker-execution-mode

### 4.3 Pull Request Title
Format: `[Dir-{ID}] {Description}`
Example:
[Dir-22] Implement Maker-Only Execution Mode

----------

## 5. Priority Levels

**Priority** **Response Time** **Review Time** **Examples** 
**P0 - Critical** Immediate < 1 hour Production down, "Gap-to-Zero" vulnerability found.
**P1 - High** < 4 hours < 4 hours Logic bug, blocking issue.
**P2 - Medium** < 1 day < 1 day Standard features, improvements.
**P3 - Low** < 1 week < 1 week Tech debt, minor optimizations.

----------

## 6. The "No Code" Rule

> [!CAUTION]
> 
> Do not write implementation code until the Directive is in ACTIVE status.

### 6.1 Rationale
-   **Focus:** Prevents wasted effort on rejected proposals.   
-   **Safety:** Ensures the Council has vetted the approach for "Ruin" risks before they enter the codebase.   
-   **Architecture:** Maintains coherence by forcing design before implementation.   

### 6.2 Exceptions
-   **Prototypes:** Exploratory code marked `[PROTOTYPE]` in branch name.    
-   **Hotfixes:** P0 incidents (must create Directive retroactively).    

----------

## 7. Version History

**Version** **Date** **Author** **Changes**
2.0.0 2025-12-27 Architect Updated for Project V (Directive-Driven).
