import json
import rust_agent_runtime

graph = rust_agent_runtime.PyGraph()

'''
에이전트 워크플로우의 state schema 정의
(이름, 기본값, 입력 시 필수 여부)
'''
graph.add_state_field("query", None, True)
graph.add_state_field("customer_id", None, True)
graph.add_state_field("docs", None, False)
graph.add_state_field("answer", None, False)

'''
노드 함수 정의
- 노드 간 state를 주고받을 때에는 항상 JSON 상태여야 함
'''
def retrieve(state_json: str) -> str:
    state = json.loads(state_json)

    return json.dumps({
        "docs": f"검색 결과 for query={state['query']}"
    }, ensure_ascii=False)

def generate(state_json: str) -> str:
    state = json.loads(state_json)

    return json.dumps({
        "answer": f"[최종 답변] {state['docs']}"
    }, ensure_ascii=False)

'''
그래프 구조 정의
'''
graph.add_node("retrieve", retrieve)
graph.add_node("generate", generate)
graph.add_edge("retrieve", "generate")

'''
초기 state 지정
'''
initial_state = json.dumps({
    "query": "안녕하세요?",
    "customer_id": "CUST001",
})

'''
에이전트 실행
'''
result = graph.run("retrieve", initial_state)
print(result)
print(json.loads(result))