From the Test Cases below, generate Playwright E2E tests.

Rules:
- Use TypeScript
- Use Page Object Model — separate page objects from test files
- Use ONLY the selectors specified in the test cases, never guess
- Every test must be independent — no test should depend on another
- Group tests with test.describe by feature/category
- Use meaningful test names that describe the behavior being verified
- Use beforeEach/afterEach for setup/teardown
- Mock APIs using page.route() with the exact endpoints 
  and responses from the test cases
- Wait for elements using Playwright's built-in auto-waiting 
  locators — never use waitForTimeout or hardcoded delays
- Assert using Playwright's expect API 
  (toBeVisible, toHaveText, toHaveURL, toBeDisabled, etc.)
- Include error handling assertions (toast messages, 
  inline errors, redirects on failure)

File structure:
tests/
  pages/
    [feature].page.ts      ← page object with selectors & actions
  [feature].spec.ts        ← test file
  fixtures/
    [feature].data.ts      ← test data constants

Test Cases:
"""
[paste Step 3 output]
"""