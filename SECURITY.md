# Security policy

## Reporting a vulnerability

Please report security problems privately, not in a public issue or pull request.

Use GitHub's private reporting: open the **Security** tab of this repository and choose
**Report a vulnerability**. The report is visible only to the maintainers.

Include what you found, how to reproduce it, and what an attacker gains. A short proof of
concept helps. Please do not include real API keys or other people's data.

You will get an acknowledgement and updates as the fix progresses. Once a fix is released, we
are happy to credit you if you wish.

## Supported versions

Fixes go into the latest release. Older releases are not patched.

## What is in scope

CleanPing handles API keys and sends your text to the AI provider you chose, so these matter most:

- an API key or your text going anywhere other than the provider you configured
  (redirects, proxies, URL parsing, the `http://` loopback exception);
- a key being printed, logged, put on a command line, or stored with loose file permissions;
- a provider reply that can affect your terminal or your shell beyond the text itself
  (control characters, extra lines from the shell key);
- the local key file, history database or shell integration being writable or readable by
  other users;
- the release process (workflows, artifacts, checksums).

Problems that need a malicious local user who already controls your account, or a provider you
deliberately configured behaving badly with the text you sent it, are out of scope.
