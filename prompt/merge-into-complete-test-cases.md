You are a QA Automation Engineer.

Merge the Test Scenarios and Technical Context below into 
complete, implementation-ready Test Cases for Playwright.

For each Test Case, specify:

1. **Test ID**: TC-XXX
2. **Title**: the exact string to use in test.describe / test()
3. **Category**: Happy / Edge / Error / Boundary
4. **Preconditions**:
   - URL to navigate to
   - State to set up (authentication, seed data, etc.)
   - APIs to mock (if any), with exact endpoint and response body
5. **Steps**: for each step provide
   - Action: click / fill / select / navigate / wait / hover
   - Target: exact selector from the Technical Context
   - Value: input value (if applicable)
6. **Assertions**: for each assertion provide
   - Type: visible / hidden / text / url / count / disabled / API-called
   - Target: exact selector or URL
   - Expected value
7. **Test Data**: concrete values used in each step
8. **API Mocks** (if needed): endpoint, method, status code, 
   response body

Do NOT invent selectors. Use only selectors from the Technical Context.
If a selector is missing, flag it explicitly.

Test Scenarios:

Technical Context:
