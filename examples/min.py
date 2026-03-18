import rust_agent_runtime
import json

graph = rust_agent_runtime.PyGraph()

def retrieve(state_json):
    state=json.loads(state_json)

    state['docs'] = "문서"

    return json.dumps(state)

def generate(state_json):
    state=json.loads(state_json)

    state['answer']="답변"

    return json.dumps(state)

graph.add_node("retrieve", retrieve)
graph.add_node("generate", generate)

graph.add_edge("retrieve", "generate")

state=json.dumps({"query":"답변은?"})

result=graph.run("retrieve", state)

print(result)