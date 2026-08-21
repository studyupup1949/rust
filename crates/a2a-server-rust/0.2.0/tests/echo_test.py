import asyncio
import json
import httpx
import sys

async def echo_test(server_url: str):
    print(f"🚀 Testing A2A Echo Skill at {server_url}")
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        try:
            # Send Echo Message via JSON-RPC
            test_message = "Hello from the test program!"
            print(f"💬 Sending message: '{test_message}'")
            
            rpc_payload = {
                "jsonrpc": "2.0",
                "method": "tasks/send",
                "params": {
                    "id": "echo-test-task",
                    "message": {
                        "role": "user",
                        "parts": [{"kind": "text", "text": test_message}],
                        "messageId": "msg-1",
                        "kind": "message"
                    }
                },
                "id": 1
            }
            
            response = await client.post(
                f"{server_url}/", 
                json=rpc_payload
            )
            
            if response.status_code == 200:
                result = response.json()
                if "error" in result:
                    print(f"❌ JSON-RPC Error: {result['error']}")
                    return

                # The response from tasks/send includes the task state
                task = result.get('result', {})
                history = task.get('history', [])
                if history:
                    # Find the last message from the agent
                    agent_msgs = [m for m in history if m.get('role') == 'agent']
                    if agent_msgs:
                        last_reply = agent_msgs[-1]
                        parts = last_reply.get('parts', [])
                        reply_text = ""
                        for part in parts:
                            if 'text' in part:
                                reply_text += part['text']
                        
                        print(f"✅ Received echo: '{reply_text}'")
                        if test_message in reply_text:
                            print("🌟 Success! The echo skill is working correctly.")
                        else:
                            print("⚠️ The response received doesn't seem to be an echo.")
                    else:
                        print("⚠️ No agent response found in history.")
                else:
                    print("⚠️ No message history returned.")
            else:
                print(f"❌ Request failed: {response.status_code}")
                print(response.text)

        except Exception as e:
            print(f"💥 Error: {str(e)}")
            if "localhost" in server_url:
                print("Is the local server running? Try 'make start' in another terminal.")

if __name__ == "__main__":
    url = "http://localhost:8080"
    if len(sys.argv) > 1:
        url = sys.argv[1]
    
    # Ensure URL doesn't end with a slash for appending / later if needed, 
    # but here we use f"{server_url}/" so it's fine.
    if url.endswith("/"):
        url = url[:-1]
        
    asyncio.run(echo_test(url))
