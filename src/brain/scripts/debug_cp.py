import aiohttp
import asyncio
import os
import json
from dotenv import load_dotenv

# Load .env manually for this script as it is standalone
load_dotenv(".env")
KEY = os.getenv("CRYPTOPANIC_API_KEY")


async def main():
    print(f"🔑 Key: {KEY[:5]}...")
    scenarios = [
        (
            "Developer V2",
            f"https://cryptopanic.com/api/developer/v2/posts/?auth_token={KEY}&kind=news&filter=hot",
        ),
        (
            "Free V1 (Retry)",
            f"https://cryptopanic.com/api/v1/posts/?auth_token={KEY}&kind=news&filter=hot",
        ),
    ]

    headers = {
        "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Accept": "application/json",
    }

    async with aiohttp.ClientSession(headers=headers) as session:
        for name, u in scenarios:
            print(f"\n--- Scenario: {name} ---")
            print(f"URL: {u.replace(KEY, 'REDACTED')}")
            async with session.get(u) as resp:
                print(f"Status: {resp.status}")
                print(f"Final URL: {str(resp.url).replace(KEY, 'REDACTED')}")
                if resp.history:
                    print(
                        f"Redirects: {[str(h.url).replace(KEY, 'REDACTED') for h in resp.history]}"
                    )

                if resp.status == 200:
                    try:
                        data = await resp.json()
                        print(f"✅ Success! Results: {len(data.get('results', []))}")
                    except:
                        print("Not JSON")
                else:
                    text = await resp.text()
                    title = (
                        text.split("<title>")[1].split("</title>")[0]
                        if "<title>" in text
                        else "No Title"
                    )
                    print(f"Error HTML Title: {title}")


if __name__ == "__main__":
    asyncio.run(main())
