# ABPilot CC SDK Testing Guide

## Full Integration Test

This guide walks you through testing the complete SDK functionality from authentication to cleanup.

## Prerequisites

1. Access to ABPilot CC platform
2. Valid email address for authentication
3. SMTP configured on the backend (for verification codes)

## Running the Full Test

### Interactive Test (Recommended for First Time)

```bash
cargo run --example full_test --all-features
```

This will:
1. Prompt for your email
2. Send verification code
3. Prompt for the code
4. Run through all SDK features
5. Clean up all created resources

### Test Coverage

The full test covers:

#### ✅ Authentication (MP)
- Send verification code
- Verify code and get JWT token
- Use token for authenticated requests

#### ✅ API Key Management (MP)
- Create API key
- List API keys
- Delete API key

#### ✅ App Management (MP)
- Create app
- List apps
- Get upload URLs for app files
- Get download URLs for app files
- Delete app

#### ✅ World Management (MP)
- Create world
- List worlds
- Get world details
- Get upload URLs for world files
- Get download URLs for world files
- Delete world

#### ✅ Device Token (APP)
- Create device token with TTL
- Get device info by token

#### ✅ World Node Management (APP)
- Add/update world nodes
- Multiple nodes with different tags
- Delete world nodes

#### ✅ Asset Operations (APP)
- List assets for device
- Add assets (gold, gems)
- Get specific asset
- Deduct assets (negative delta)
- Test insufficient balance error

#### ✅ Cleanup
- Delete all created resources
- Verify cleanup

## Expected Output

```
=== ABPilot CC SDK End-to-End Test ===

📧 Step 1: Email Authentication
Enter your email: user@example.com
Sending verification code to user@example.com...
✅ Verification code sent!

Enter the 6-digit code from your email: 123456
Verifying code...
✅ Authentication successful!
   User ID: abc12345
   Token: eyJhbGciOiJIUzI1NiI...

🔑 Step 2: API Key Management
Creating API key...
✅ API key created: sk_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
Listing all API keys...
✅ Found 1 API key(s):
   - Test API Key (sk_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx)

📱 Step 3: App Management
Creating app...
✅ App created!
   App ID: rImLACph7Ayr8tu1
   Name: Test Game
   Secret: 5d75a1ee1cd34b2e9122b44d36ddf798
...

💰 Step 7: Asset Operations
Listing assets for device test_device_001...
✅ Found 0 asset(s)

Adding 100 gold...
✅ Gold added! New balance: 100

Adding 50 more gold...
✅ Gold added! New balance: 150

Adding 200 gems...
✅ Gems added! Balance: 200

Getting specific asset (gold)...
✅ Gold balance: 150

Deducting 30 gold...
✅ Gold deducted! New balance: 120

Listing all assets again...
✅ Current assets:
   - gold 001 = 120
   - gem 001 = 200

⚠️  Step 9: Testing Insufficient Balance
Attempting to deduct 1000 gold (should fail)...
✅ Correctly failed: Insufficient balance

🧹 Step 10: Cleanup
Deleting world node 1...
✅ World node 1 deleted
...

✅ ========================================
✅ All tests completed successfully!
✅ ========================================
```

## Testing Individual Features

### Test MP Features Only

```bash
# Authentication
cargo run --example auth_flow --features mp

# App management
ABPILOT_TOKEN=your_jwt_token cargo run --example app_management --features mp

# World management
ABPILOT_TOKEN=your_jwt_token cargo run --example world_management --features mp
```

### Test APP Features Only

```bash
# Asset operations
APP_ID=your_app_id \
APP_SECRET=your_app_secret \
WORLD_ID=your_world_id \
WORLD_SECRET=your_world_secret \
cargo run --example asset_operations --features app
```

## Unit Tests

Run all unit tests:

```bash
# All features
cargo test --all-features

# MP only
cargo test --no-default-features --features mp

# APP only
cargo test --no-default-features --features app
```

## Troubleshooting

### Verification Code Not Received

- Check SMTP configuration on backend
- Check spam folder
- Verify email address is correct
- Code expires in 5 minutes

### Authentication Failed

- Ensure code is entered within 5 minutes
- Code is case-sensitive (if applicable)
- Check for typos

### API Errors

- Check network connectivity
- Verify Lambda URLs are accessible
- Check authentication token/API key is valid
- Review error messages for details

### Insufficient Balance Error

This is expected when trying to deduct more than available:
```
✅ Correctly failed: Insufficient balance
```

### Token Expired

Device tokens have TTL (default 3600 seconds). Create a new token if expired.

## Performance Testing

For load testing, consider:

1. Multiple concurrent requests
2. Large asset operations
3. Many world nodes
4. Bulk device token creation

Example:
```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();
for i in 0..100 {
    let client = client.clone();
    set.spawn(async move {
        client.app().list_assets(...).await
    });
}
```

## Security Testing

Verify:
- ✅ Invalid signatures are rejected
- ✅ Expired tokens are rejected
- ✅ Unauthorized access is blocked
- ✅ Secrets are not logged

## CI/CD Integration

Add to your CI pipeline:

```yaml
# .github/workflows/test.yml
- name: Run SDK tests
  run: |
    cargo test --all-features
    cargo clippy --all-features -- -D warnings
    cargo fmt -- --check
```

## Next Steps

After successful testing:

1. Review the code in `examples/` for usage patterns
2. Check `USAGE.md` for API reference
3. Integrate SDK into your application
4. Set up proper error handling
5. Configure logging/monitoring

## Support

For issues or questions:
- Check error messages carefully
- Review API documentation (MP.md, APP.md)
- Examine example code
- Check integration test output
