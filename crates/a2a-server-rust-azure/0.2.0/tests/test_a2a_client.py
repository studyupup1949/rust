import asyncio
import json
import httpx

async def test_a2a_server():
    print("🧪 Validating A2A Server Discovery...")
    server_url = "http://localhost:8080"
    
    async with httpx.AsyncClient() as client:
        try:
            # 1. Test Discovery Endpoint (GET /agent-card or /.well-known/agent-card.json)
            response = await client.get(f"{server_url}/agent-card")
            if response.status_code == 200:
                print("✅ Discovery endpoint reachable")
                info = response.json()
                print(f"📄 Agent Name: {info.get('name')}")
                print(f"📄 Skills: {[s.get('name') for s in info.get('skills', [])]}")
            else:
                print(f"❌ Discovery endpoint failed: {response.status_code}")
                return

            # 2. Test Message Processing via JSON-RPC (POST / with method tasks/send)
            print("\n🧪 Sending Echo Message via JSON-RPC...")
            test_message = "Hello Rust A2A Server! Echo this."
            rpc_payload = {
                "jsonrpc": "2.0",
                "method": "tasks/send",
                "params": {
                    "id": "test-task-python",
                    "message": {
                        "role": "user",
                        "parts": [{"kind": "text", "text": test_message}],
                        "messageId": "msg-python-1",
                        "kind": "message"
                    }
                },
                "id": 1
            }
            
            response = await client.post(f"{server_url}/", json=rpc_payload)
            
            if response.status_code == 200:
                result = response.json()
                if "error" in result:
                    print(f"❌ JSON-RPC Error: {result['error']}")
                    return
                
                print("✅ Message processed")
                task = result.get('result', {})
                print(f"🔄 Task State: {task.get('status', {}).get('state')}")
                
                # Check for echo response in history
                history = task.get('history', [])
                if history:
                    agent_msgs = [m for m in history if m.get('role') == 'agent']
                    if agent_msgs:
                        last_msg = agent_msgs[-1]
                        print(f"🤖 Agent Response: {json.dumps(last_msg, indent=2)}")
                    else:
                        print("⚠️ No agent response found in history.")
                else:
                    print("⚠️ No message history returned in the response.")
            else:
                print(f"❌ Message processing failed: {response.status_code}")
                print(response.text)

        except Exception as e:
            print(f"💥 Error during validation: {str(e)}")

if __name__ == "__main__":
    asyncio.run(test_a2a_server())
