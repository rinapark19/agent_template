import json
import pulp
import rust_agent_runtime

from agent_template.utils.runtime_registry import registry_session, registry_managed_node, dumps_state_for_runtime

graph = rust_agent_runtime.PyGraph()

graph.add_state_field("query", None, True)
graph.add_state_field("problem", None, False)
graph.add_state_field("x", None, False)
graph.add_state_field("answer", '""', False)

@registry_managed_node
def build_problem(state: dict) -> dict:
    x = pulp.LpVariable("x", lowBound=0)
    prob = pulp.LpProblem("demo", pulp.LpMaximize)
    prob += x
    return {"problem": prob, "x": x}

@registry_managed_node
def solve_problem(state: dict) -> dict:
    prob = state['problem']
    prob.solve()
    return {"answer": "done"}

graph.add_node("build_problem", build_problem)
graph.add_node("solve_problem", solve_problem)
graph.add_edge("build_problem", "solve_problem")

with registry_session():
    result_json = graph.run(
        "build_problem",
        dumps_state_for_runtime({"query": "optimize"}),
    )
    print(json.loads(result_json))