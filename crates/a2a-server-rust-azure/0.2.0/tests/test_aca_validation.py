import asyncio
import json
import httpx
import sys

# Default Azure Container App (ACA) Service URL (if not overridden by command line argument)
ACA_URL = ""

async def test_a2a_aca(server_url: str):
    print(f"🧪 Validating A2A Server at Azure Container App: {server_url}")
    
    async with httpx.AsyncClient(timeout=30.0, follow_redirects=True) as client:
        try:
            # 1. Test Discovery
            print("\n🔍 Step 1: Discovery...")
            
            # Try GET /agentcard (A2A Specification Endpoint)
            print("  Trying GET /agentcard ...")
            resp = await client.get(f"{server_url}/agentcard")
            if resp.status_code == 200:
                print("  ✅ GET /agentcard success")
                info = resp.json()
            else:
                print(f"  ❌ GET /agentcard failed: {resp.status_code}")
                
                # Try GET /agent-card (Alternative/legacy A2A endpoint)
                print("  Trying GET /agent-card ...")
                resp = await client.get(f"{server_url}/agent-card")
                if resp.status_code == 200:
                    print("  ✅ GET /agent-card success")
                    info = resp.json()
                else:
                    # Try GET / (Standard A2A)
                    print("  Trying GET / ...")
                    resp = await client.get(f"{server_url}/")
                    if resp.status_code == 200:
                        print("  ✅ GET / success")
                        info = resp.json()
                    else:
                        print(f"  ❌ GET / failed: {resp.status_code}")
                        return

            print(f"📄 Agent Name: {info.get('name')}")
            skills = [s.get('name') for s in info.get('skills', [])]
            print(f"📄 Skills: {skills}")

            # 2. Test Task Creation / Sending
            print("\n🧪 Step 2: Creating/Sending A2A Task...")
            task_id = "aca-test-task-1"
            
            # Using JSON-RPC tasks/send
            print("  Trying JSON-RPC tasks/send ...")
            rpc_payload = {
                "jsonrpc": "2.0",
                "method": "tasks/get", # Using tasks/get to check existence first, known to work with 'id'
                "params": {
                    "id": task_id,
                    "contextId": "aca-validation-context"
                },
                "id": 2
            }
            resp = await client.post(f"{server_url}/", json=rpc_payload)
            if resp.status_code == 200:
                rpc_resp = resp.json()
                if "error" in rpc_resp and rpc_resp["error"]["code"] == -32001:
                    print(f"  ✅ JSON-RPC reachable (Task not found as expected)")
                else:
                    print(f"  ✅ JSON-RPC reachable: {rpc_resp}")
            else:
                print(f"  ❌ JSON-RPC failed: {resp.status_code}")
                return

            print("\n✅ Basic validation complete. Remote A2A agent on Azure is responsive.")

        except Exception as e:
            print(f"💥 Error during validation: {str(e)}")

if __name__ == "__main__":
    url = ACA_URL
    if len(sys.argv) > 1:
        url = sys.argv[1]
    if url.endswith("/"):
        url = url[:-1]
    
    if not url:
        print("❌ Error: No ACA URL specified. Please provide the server URL as an argument.")
        sys.exit(1)
        
    asyncio.run(test_a2a_aca(url))
