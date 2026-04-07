# Security Policy

## 1. Supported Versions

We are committed to providing security updates for the most recent versions of Geminiscope. Please see the table below for the current support status:

| Version | Supported          |
| ------- | ------------------ |
| v0.2.x  | ✅ Yes              |
| v0.1.x  | ❌ No               |
| < v0.1  | ❌ No               |

## 2. Reporting a Vulnerability

**DO NOT OPEN A PUBLIC GITHUB ISSUE for security vulnerabilities.**

If you discover a security vulnerability in Geminiscope, please report it privately to ensure the safety of our users. We appreciate your responsible disclosure and will work with you to resolve the issue promptly.

### How to Report
Please send an email to **raunak.jyotishi@gmail.com** with the following information:

1.  **Summary**: A brief description of the vulnerability.
2.  **Severity**: Your assessment of the impact (Low, Medium, High, Critical).
3.  **Proof of Concept (PoC)**: Detailed steps, scripts, or screenshots to reproduce the issue.
4.  **Affected Component**: Which part of the TUI or Parser is impacted.

We encourage the use of PGP encryption for sensitive reports. Please request our public key via email if you wish to use it.

## 3. Response Process

Upon receiving your report, you can expect the following timeline:

*   **Initial Acknowledgment**: Within **48 hours** of receipt.
*   **Triage & Validation**: Within **5 business days**.
*   **Resolution/Fix**: Timeline will vary based on complexity, but we aim for a resolution within **30 days**.

### Public Disclosure
We ask that you maintain strict confidentiality and observe an **embargo period** until a fix has been released and users have had a reasonable time to update. We will coordinate with you on a public disclosure date once the vulnerability is mitigated.

## 4. Out of Scope

The following types of attacks and issues are strictly out of scope for our vulnerability disclosure program:

*   **Volumetric Attacks**: Distributed Denial of Service (DDoS) or other resource exhaustion attacks.
*   **Social Engineering**: Attacks targeting project maintainers or users (phishing, vishing, etc.).
*   **Physical Attacks**: Any attack requiring physical access to hardware or infrastructure.
*   **Third-Party Dependencies**: Vulnerabilities in upstream libraries (e.g., `ratatui`, `tokio`) should be reported directly to their respective maintainers, though we appreciate being informed if it affects Geminiscope's security posture.
*   **Non-Security Bugs**: General UI glitches or crashes that do not have a security impact.

---
*Thank you for helping keep Geminiscope secure!*
