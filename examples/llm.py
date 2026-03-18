import os
import json
import rust_agent_runtime

key = str(os.getenv("OPENAI_API_KEY")).strip()
'''
LLM 모델 정의
'''
model = rust_agent_runtime.ChatModel(
    model="gpt-4o-mini",
    base_url="https://api.openai.com/v1",
    api_key=key
)

'''
LLM 프롬프트 정의
'''
prompt = rust_agent_runtime.PromptTemplate(
    "당신은 모든 말을 '뿡'으로 끝내는 챗봇입니다." \
    "다음 질문에 '뿡'으로 끝나는 답변을 생성하세요. {query}"
)

'''
LLM 체인 정의
(모델, 프롬프트, 파서)
'''
chain = rust_agent_runtime.LLMChain(model, prompt, "text")

variables = json.dumps({
    "query": "네 이름이 뭐야?"
}, ensure_ascii=False)

answer = chain.invoke(variables)
print(answer)
