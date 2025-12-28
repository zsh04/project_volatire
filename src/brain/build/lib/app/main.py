import uvicorn
from fastapi import FastAPI
import logging

# Initialize Logger
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("Brain")

app = FastAPI(title="Voltaire Brain", version="0.1.0")

@app.on_event("startup")
async def startup_event():
    logger.info("The Brain (Mind) is Waking Up...")

@app.get("/health")
async def health_check():
    return {"status": "conscious", "role": "Mind"}

def main():
    """Entrypoint for the Brain service."""
    logger.info("Igniting Neural Pathways...")
    uvicorn.run("src.brain.app.main:app", host="0.0.0.0", port=8000, reload=True)

if __name__ == "__main__":
    main()
