#!/bin/bash

echo "=== ABPilot CC SDK 快速测试 ==="
echo ""

# 检查是否有 token
if [ -n "$ABPILOT_TOKEN" ]; then
    echo "✅ 检测到 ABPILOT_TOKEN"
    echo "运行完整测试..."
    cargo run --example full_test_with_token --all-features
    exit 0
fi

if [ -n "$ABPILOT_API_KEY" ]; then
    echo "✅ 检测到 ABPILOT_API_KEY"
    echo "运行完整测试..."
    cargo run --example full_test_with_token --all-features
    exit 0
fi

echo "❌ 未检测到认证信息"
echo ""
echo "请设置以下环境变量之一："
echo "  export ABPILOT_TOKEN=\"your_jwt_token\""
echo "  export ABPILOT_API_KEY=\"sk_your_api_key\""
echo ""
echo "或者尝试获取 token（需要邮件验证）："
echo "  cargo run --example get_token --all-features"
echo ""
echo "注意：当前后端 SMTP 未配置，邮件验证可能无法使用"
echo "建议联系管理员获取测试 token"

exit 1
