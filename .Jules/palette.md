## 2024-05-23 - Accessibility in High-Density Interfaces
**Learning:** In data-heavy "tactical" interfaces like this trading dashboard, icon-only buttons are common for space efficiency but often lack accessible labels. Adding `aria-label` and `title` is a high-impact, low-effort win. Also, toggles using color-only state need `aria-pressed` to be perceivable by screen readers.
**Action:** Always check icon-only toolbars for `aria-label` and verify state toggles have semantic attributes, not just color changes.
