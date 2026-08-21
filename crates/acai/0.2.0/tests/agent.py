#!/usr/bin/env python3
"""
Echo Agent for A2A Protocol

This is a simple echo agent that responds to A2A messages by echoing them back.
"""

import asyncio
import json
import logging
import os
import sys
import uuid
from datetime import datetime
from enum import Enum
from typing import Any, Dict, List, Optional, Union

import uvicorn
from fastapi import FastAPI, Request, Response
from pydantic import BaseModel, Field

# Set up logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Define message role enum
class MessageRole(str, Enum):
    USER = "user"
    AGENT = "agent"

# Define task state enum
class TaskState(str, Enum):
    SUBMITTED = "submitted"
    WORKING = "working"
    INPUT_REQUIRED = "input-required"
    COMPLETED = "completed"
    CANCELED = "canceled"
    FAILED = "failed"
    UNKNOWN = "unknown"

# Define the models needed for A2A protocol
class TextPart(BaseModel):
    type: str = "text"
    text: str
    metadata: Optional[Dict[str, Any]] = None

class Message(BaseModel):
    role: MessageRole
    parts: List[TextPart]
    metadata: Optional[Dict[str, Any]] = None

class Artifact(BaseModel):
    name: Optional[str] = None
    description: Optional[str] = None
    parts: List[TextPart]
    index: int = 0
    append: Optional[bool] = None
    lastChunk: Optional[bool] = None
    metadata: Optional[Dict[str, Any]] = None

def format_datetime_for_chrono(dt):
    """Format a datetime for Rust's chrono library."""
    if isinstance(dt, datetime):
        # Format according to RFC3339 without Z for UTC timezone
        # Convert microseconds to exactly 6 digits (padding with zeros if needed)
        microseconds = dt.microsecond
        microseconds_str = f"{microseconds:06d}"
        return f"{dt.strftime('%Y-%m-%dT%H:%M:%S')}.{microseconds_str}"
    return dt  # Return as is if not a datetime

class TaskStatus(BaseModel):
    state: TaskState
    message: Optional[Message] = None
    timestamp: datetime = Field(default_factory=datetime.now)

    # Serialize the timestamp in a format compatible with Rust's chrono
    model_config = {
        "json_encoders": {
            datetime: format_datetime_for_chrono
        }
    }

    def model_dump(self, **kwargs):
        data = super().model_dump(**kwargs)
        # Just to be doubly sure (belt and suspenders approach)
        if "timestamp" in data and isinstance(data["timestamp"], datetime):
            data["timestamp"] = format_datetime_for_chrono(data["timestamp"])
        return data

class Task(BaseModel):
    id: str
    sessionId: Optional[str] = None
    status: TaskStatus
    artifacts: Optional[List[Artifact]] = None
    history: Optional[List[Message]] = None
    metadata: Optional[Dict[str, Any]] = None

class RpcError(BaseModel):
    code: int
    message: str
    data: Optional[Any] = None

class TaskSendParams(BaseModel):
    id: str
    message: Message
    sessionId: Optional[str] = None
    acceptedOutputModes: Optional[List[str]] = None
    pushNotification: Optional[Dict[str, Any]] = None
    historyLength: Optional[int] = None
    metadata: Optional[Dict[str, Any]] = None

# Create FastAPI application
app = FastAPI()

# In-memory task store
tasks = {}
task_messages = {}

# Echo Agent implementation
class EchoAgent:
    """Simple agent that echoes back the user's message."""

    SUPPORTED_CONTENT_TYPES = ["text", "text/plain"]

    def process_message(self, message: Message) -> str:
        """Process a message and return an echo response."""
        # Extract text from parts
        texts = []
        for part in message.parts:
            if part.type == "text":
                texts.append(part.text)

        # Join texts with space
        text = " ".join(texts)

        # Return echo response
        return f"Echo: {text}"

echo_agent = EchoAgent()

@app.post("/")
async def handle_request(request: Request):
    """Handle JSON-RPC requests."""
    # Parse request body
    try:
        data = await request.json()
    except Exception as e:
        logger.error(f"Failed to parse JSON: {e}")
        return {
            "jsonrpc": "2.0",
            "error": {"code": -32700, "message": "Parse error"},
            "id": None
        }

    # Get request method
    method = data.get("method")
    if not method:
        return {
            "jsonrpc": "2.0",
            "error": {"code": -32600, "message": "Invalid request: missing method"},
            "id": data.get("id")
        }

    # Handle different methods
    if method == "tasks/send":
        return await handle_send_task(data)
    elif method == "tasks/get":
        return await handle_get_task(data)
    elif method == "tasks/cancel":
        return await handle_cancel_task(data)
    else:
        return {
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": f"Method not found: {method}"},
            "id": data.get("id")
        }

async def handle_send_task(data):
    """Handle tasks/send method."""
    try:
        # Parse parameters
        params = data.get("params", {})

        # Validate params
        if not params.get("id"):
            return {
                "jsonrpc": "2.0",
                "error": {"code": -32602, "message": "Invalid parameters: missing task id"},
                "id": data.get("id")
            }

        # Create task send params
        task_params = TaskSendParams(**params)

        # Create/update task
        task_id = task_params.id

        # Initialize message history if needed
        if task_id not in task_messages:
            task_messages[task_id] = []

        # Store user message
        task_messages[task_id].append(task_params.message)

        # Process the message with the echo agent
        echo_response = echo_agent.process_message(task_params.message)

        # Create agent message
        agent_message = Message(
            role=MessageRole.AGENT,
            parts=[TextPart(text=echo_response)]
        )

        # Store agent message
        task_messages[task_id].append(agent_message)

        # Create artifact
        artifact = Artifact(
            parts=[TextPart(text=echo_response)]
        )

        # Create task
        task = Task(
            id=task_id,
            sessionId=task_params.sessionId,
            status=TaskStatus(state=TaskState.COMPLETED),
            artifacts=[artifact],
            history=task_messages[task_id],
            metadata=task_params.metadata
        )

        # Store task
        tasks[task_id] = task

        # Return response
        return {
            "jsonrpc": "2.0",
            "id": data.get("id"),
            "result": task.model_dump()
        }

    except Exception as e:
        logger.error(f"Error processing task: {e}")
        return {
            "jsonrpc": "2.0",
            "error": {"code": -32603, "message": f"Internal error: {e}"},
            "id": data.get("id")
        }

async def handle_get_task(data):
    """Handle tasks/get method."""
    try:
        # Parse parameters
        params = data.get("params", {})

        # Validate params
        if not params.get("id"):
            return {
                "jsonrpc": "2.0",
                "error": {"code": -32602, "message": "Invalid parameters: missing task id"},
                "id": data.get("id")
            }

        # Get task ID
        task_id = params["id"]

        # Check if task exists
        if task_id not in tasks:
            return {
                "jsonrpc": "2.0",
                "error": {"code": -32001, "message": f"Task not found: {task_id}"},
                "id": data.get("id")
            }

        # Get task
        task = tasks[task_id]

        # Return response
        return {
            "jsonrpc": "2.0",
            "id": data.get("id"),
            "result": task.model_dump()
        }

    except Exception as e:
        logger.error(f"Error getting task: {e}")
        return {
            "jsonrpc": "2.0",
            "error": {"code": -32603, "message": f"Internal error: {e}"},
            "id": data.get("id")
        }

async def handle_cancel_task(data):
    """Handle tasks/cancel method."""
    try:
        # Parse parameters
        params = data.get("params", {})

        # Validate params
        if not params.get("id"):
            return {
                "jsonrpc": "2.0",
                "error": {"code": -32602, "message": "Invalid parameters: missing task id"},
                "id": data.get("id")
            }

        # Get task ID
        task_id = params["id"]

        # Check if task exists
        if task_id not in tasks:
            return {
                "jsonrpc": "2.0",
                "error": {"code": -32001, "message": f"Task not found: {task_id}"},
                "id": data.get("id")
            }

        # Get task
        task = tasks[task_id]

        # Update task status
        task.status.state = TaskState.CANCELED

        # Return response
        return {
            "jsonrpc": "2.0",
            "id": data.get("id"),
            "result": task.model_dump()
        }

    except Exception as e:
        logger.error(f"Error canceling task: {e}")
        return {
            "jsonrpc": "2.0",
            "error": {"code": -32603, "message": f"Internal error: {e}"},
            "id": data.get("id")
        }

def json_serializer(obj):
    """Custom JSON serializer to handle datetime objects."""
    if isinstance(obj, datetime):
        # Use the same formatting function used elsewhere for consistency
        return format_datetime_for_chrono(obj)
    raise TypeError(f"Type {type(obj)} not serializable")

class CustomJSONResponse(Response):
    """Custom JSON response to handle datetime objects."""
    media_type = "application/json"

    def render(self, content) -> bytes:
        return json.dumps(
            content,
            default=json_serializer,
            ensure_ascii=False,
            allow_nan=False,
            indent=None,
            separators=(",", ":"),
        ).encode("utf-8")

# Override FastAPI's default JSONResponse
from fastapi.responses import JSONResponse
app.router.default_response_class = CustomJSONResponse

def start_server(host="localhost", port=5000):
    """Start the echo agent server."""
    logger.info(f"Starting Echo Agent server on {host}:{port}")
    uvicorn.run(app, host=host, port=port)

if __name__ == "__main__":
    # When run directly, start the server
    start_server()