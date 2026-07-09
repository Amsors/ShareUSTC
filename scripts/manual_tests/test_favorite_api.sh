#!/bin/bash

# 收藏夹功能 API 测试脚本
# 使用方法：
# 1. 先登录获取 token
# 2. 设置 TOKEN 环境变量
# export TOKEN="your_jwt_token"
# 3. 运行脚本：bash test_favorite_api.sh

BASE_URL="http://localhost:8080/api"

echo "========================================="
echo "ShareUSTC 收藏夹功能 API 测试"
echo "========================================="
echo ""

if [ -z "$TOKEN" ]; then
    echo "错误：请设置 TOKEN 环境变量"
    echo "export TOKEN=\"your_jwt_token\""
    exit 1
fi

# 1. 创建收藏夹
echo "1. 创建收藏夹"
echo "----------------------------------------"
CREATE_RESPONSE=$(curl -s -X POST "${BASE_URL}/favorites" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${TOKEN}" \
  -d '{"name": "测试收藏夹"}')
echo "响应: $CREATE_RESPONSE"
echo ""

# 2. 获取收藏夹列表
echo "2. 获取收藏夹列表"
echo "----------------------------------------"
curl -s -X GET "${BASE_URL}/favorites" \
  -H "Authorization: Bearer ${TOKEN}" | jq .
echo ""

# 3. 获取收藏夹详情（需要替换为实际的收藏夹ID）
echo "3. 获取收藏夹详情"
echo "----------------------------------------"
echo "请提供收藏夹ID:"
read FAVORITE_ID
if [ ! -z "$FAVORITE_ID" ]; then
    curl -s -X GET "${BASE_URL}/favorites/${FAVORITE_ID}" \
      -H "Authorization: Bearer ${TOKEN}" | jq .
fi
echo ""

# 4. 添加资源到收藏夹（需要替换为实际的资源ID）
echo "4. 添加资源到收藏夹"
echo "----------------------------------------"
echo "请提供资源ID:"
read RESOURCE_ID
if [ ! -z "$FAVORITE_ID" ] && [ ! -z "$RESOURCE_ID" ]; then
    curl -s -X POST "${BASE_URL}/favorites/${FAVORITE_ID}/resources" \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer ${TOKEN}" \
      -d "{\"resourceId\": \"${RESOURCE_ID}\"}" | jq .
fi
echo ""

# 5. 检查资源收藏状态
echo "5. 检查资源收藏状态"
echo "----------------------------------------"
if [ ! -z "$RESOURCE_ID" ]; then
    curl -s -X GET "${BASE_URL}/favorites/check/${RESOURCE_ID}" \
      -H "Authorization: Bearer ${TOKEN}" | jq .
fi
echo ""

echo "========================================="
echo "测试完成"
echo "========================================="
