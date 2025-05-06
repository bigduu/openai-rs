import openai
import sys

def chat_with_server(message):
    # 配置OpenAI客户端
    client = openai.OpenAI(
        base_url="http://127.0.0.1:8080/v1",  
        api_key="dummy"  
    )
    use_stream = True
    
    try:
        # 创建聊天完成请求
        stream = client.chat.completions.create(
            model="Pro/deepseek-ai/DeepSeek-V3",  
            messages=[
                {"role": "user", "content": message}
            ],
            stream=use_stream
        )

        if use_stream:
            for chunk in stream:
                if chunk.choices[0].delta.content is not None:
                    print(chunk.choices[0].delta.content, end='', flush=True)
            print()  # 换行
        else:
            print(stream.choices[0].message.content)
        print()  # 换行
        
        
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":

    chat_with_server("test")
    # if len(sys.argv) > 1:
    #     # message = " ".join(sys.argv[1:])
    #     chat_with_server("test")
    # else:
    #     print("Please provide a message to send to the server.")
    #     print("Usage: python chat_client.py 'Your message here'") 