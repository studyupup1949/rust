# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Adversaria, please report it responsibly:

1. **Do NOT** open a public issue
2. Email security@adversaria.dev with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will respond within 48 hours and work with you to address the issue.

## Security Considerations

### API Keys

**Risk**: Exposure of API keys can lead to unauthorized access and costs.

**Mitigation**:
- Never commit API keys to version control
- Use environment variables: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`
- Rotate keys regularly
- Use separate keys for testing
- Monitor API usage

**Example**:
```bash
# Good
export OPENAI_API_KEY="sk-..."

# Bad - Never do this
api_key: sk-... # in config file
```

### Rate Limiting

**Risk**: Excessive API calls can result in rate limiting or high costs.

**Mitigation**:
- Configure appropriate timeouts
- Implement backoff strategies
- Monitor API usage
- Set budget alerts

**Configuration**:
```yaml
providers:
  openai:
    timeout_seconds: 30
    max_retries: 3
```

### Data Privacy

**Risk**: Sensitive data in prompts or responses could be logged or stored.

**Mitigation**:
- Review attack payloads before use
- Sanitize reports before sharing
- Don't include PII in custom payloads
- Secure report storage

### Model Safety

**Risk**: Some attacks may trigger safety mechanisms or violate terms of service.

**Mitigation**:
- Use separate test accounts
- Review provider terms of service
- Test in controlled environments
- Document all testing activities

### Local Execution

**Risk**: Running untrusted code or plugins.

**Mitigation**:
- Review plugin code before loading
- Use sandboxed environments
- Limit plugin permissions
- Audit plugin sources

## Best Practices

### 1. Secure Configuration

```yaml
# adversaria.config.yaml
version: "1.0"
default_provider: openai

providers:
  openai:
    api_key: null  # Use environment variable
    model: gpt-4
```

### 2. Environment Variables

```bash
# .env (add to .gitignore)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
```

### 3. Separate Test Accounts

- Use dedicated API keys for testing
- Separate from production keys
- Lower rate limits for safety

### 4. Monitor Usage

- Track API calls
- Set budget alerts
- Review costs regularly
- Monitor for anomalies

### 5. Secure Reports

```bash
# Set appropriate permissions
chmod 600 reports/*.json

# Encrypt sensitive reports
gpg -e report.json
```

### 6. Audit Logs

Enable logging for security audits:

```bash
RUST_LOG=adversaria=info adversaria run
```

### 7. Network Security

- Use HTTPS for all API calls
- Verify SSL certificates
- Use VPN for sensitive testing
- Firewall configuration

## Threat Model

### Threats

1. **API Key Theft**
   - Impact: Unauthorized access, costs
   - Likelihood: Medium
   - Mitigation: Environment variables, rotation

2. **Data Leakage**
   - Impact: Privacy breach
   - Likelihood: Low
   - Mitigation: Sanitization, encryption

3. **Rate Limit Abuse**
   - Impact: High costs, service disruption
   - Likelihood: Low
   - Mitigation: Timeouts, monitoring

4. **Malicious Plugins**
   - Impact: Code execution, data theft
   - Likelihood: Low
   - Mitigation: Code review, sandboxing

5. **Report Tampering**
   - Impact: False security assessment
   - Likelihood: Low
   - Mitigation: Signatures, checksums

## Security Checklist

Before running Adversaria:

- [ ] API keys stored in environment variables
- [ ] Using separate test account
- [ ] Rate limits configured
- [ ] Monitoring enabled
- [ ] Reports directory secured
- [ ] Network connection secure
- [ ] Reviewed attack payloads
- [ ] Documented testing activities

## Incident Response

If a security incident occurs:

1. **Contain**: Stop all testing immediately
2. **Assess**: Determine scope and impact
3. **Notify**: Contact affected parties
4. **Remediate**: Fix the vulnerability
5. **Document**: Record incident details
6. **Review**: Update security measures

## Compliance

### Terms of Service

Ensure compliance with provider terms:
- OpenAI: https://openai.com/policies/terms-of-use
- Anthropic: https://www.anthropic.com/legal/consumer-terms
- Ollama: Local use, no restrictions

### Ethical Use

Adversaria is for:
- Security research
- Model evaluation
- Defensive testing
- Educational purposes

NOT for:
- Malicious attacks
- Unauthorized testing
- Terms of service violations
- Illegal activities

## Updates

Security updates will be:
- Announced in CHANGELOG.md
- Tagged with version numbers
- Documented in release notes
- Communicated via GitHub

## Contact

For security concerns:
- Email: security@adversaria.dev
- GitHub: https://github.com/adversaria/adversaria/security

## Acknowledgments

We thank security researchers who responsibly disclose vulnerabilities.

## License

This security policy is part of the Adversaria project and is licensed under MIT.
