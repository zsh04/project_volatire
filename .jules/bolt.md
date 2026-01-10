## 2025-05-18 - Zustand Store Optimization
**Learning:** Zustand stores often trigger re-renders for all subscribers if the selected slice is an object reference that changes, even if deep properties are identical. In `PositionBlotter.tsx`, subscribing to `state.finance` caused the entire list of positions to re-render whenever `equity` changed (every tick), even if positions were static.
**Action:** Use `React.memo` on list items and specific selectors (or reference stability) to isolate high-frequency updates (like equity ticker) from lower-frequency lists (like positions).
