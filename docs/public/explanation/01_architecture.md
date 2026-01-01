# Explanation: The Bi-Cameral Architecture

**Concept:** Separation of "Survival" and "Intelligence".

## The Reflex (Rust)

The "Body" or "Lizard Brain". It handles:

* **Physics:** Calculating velocity, jerk, and entropy of price action.
* **Risk:** The "Iron Gate" that prevents ruin.
* **Execution:** Sending orders to the exchange.
* **Constraint:** Must operate in $O(1)$ time.

## The Brain (Python)

The "Mind" or "Consciousness". It handles:

* **Strategy:** "Boyd" decides *what* to do.
* **Forecasting:** "Kepler" predicts *where* price might go (Probabilistic).
* **Context:** "Hypatia" understands *why* (News/Sentiment).

## The Interaction

They communicate via **gRPC** over Unix Domain Sockets. The Reflex requires a "Heartbeat" every 500ms. If the Brain is silent (crashed or GC pause), the Reflex assumes "Brain Death" and halts all trading to protect capital.
