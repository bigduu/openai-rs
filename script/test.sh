#!/bin/bash

curl --request POST \
  --url http://127.0.0.1:8080/v1/chat/completions \
  --header 'Authorization: Bearer <token>' \
  --header 'Content-Type: application/json' \
  --data '{
  "model": "Qwen/Qwen2.5-VL-72B-Instruct",
  "stream": false,
  "max_tokens": 512,
  "enable_thinking": true,
  "thinking_budget": 512,
  "min_p": 0.05,
  "temperature": 0.7,
  "top_p": 0.7,
  "top_k": 50,
  "frequency_penalty": 0.5,
  "n": 1,
  "stop": [],
  "messages": [
    {
      "role": "user",
      "content": "test"
    }
  ]
}'