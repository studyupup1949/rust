# Quick Start - Running Full Test

## 1. Build the SDK

```bash
cd sdk
cargo build --all-features
```

## 2. Run the Full Integration Test

```bash
cargo run --example full_test --all-features
```

## 3. Follow the Prompts

```
Enter your email: your@email.com
```

Check your email for the verification code.

```
Enter the 6-digit code from your email: 123456
```

## 4. Watch the Test Run

The test will automatically:
- ✅ Authenticate with email/code
- ✅ Create API key
- ✅ Create app and world
- ✅ Generate device token
- ✅ Add world nodes
- ✅ Perform asset operations (add gold, gems)
- ✅ Test error handling
- ✅ Clean up all resources

## Expected Duration

~30-60 seconds (depending on network)

## What You'll See

```
=== ABPilot CC SDK End-to-End Test ===

📧 Step 1: Email Authentication
✅ Verification code sent!
✅ Authentication successful!

🔑 Step 2: API Key Management
✅ API key created

📱 Step 3: App Management
✅ App created!

🌍 Step 4: World Management
✅ World created!

📲 Step 5: Device Token Creation
✅ Device token created

🖥️  Step 6: World Node Management
✅ World node added

💰 Step 7: Asset Operations
✅ Gold added! New balance: 100
✅ Gold added! New balance: 150
✅ Gems added! Balance: 200
✅ Gold deducted! New balance: 120

⚠️  Step 9: Testing Insufficient Balance
✅ Correctly failed: Insufficient balance

🧹 Step 10: Cleanup
✅ All resources deleted

✅ All tests completed successfully!
```

## Troubleshooting

**Code not received?**
- Check spam folder
- Wait 1-2 minutes
- Try again

**Authentication failed?**
- Code expires in 5 minutes
- Check for typos
- Request new code

**Network errors?**
- Check internet connection
- Verify Lambda URLs are accessible

## Next Steps

After successful test:
1. Review `USAGE.md` for API documentation
2. Check `examples/` for code samples
3. Read `TESTING.md` for detailed test guide
4. Start integrating into your app!

## Quick Commands

```bash
# Run full test
cargo run --example full_test --all-features

# Run unit tests
cargo test --all-features

# Build release
cargo build --release --all-features

# Check code
cargo clippy --all-features

# Format code
cargo fmt
```
