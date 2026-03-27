pub fn extract_test_scenarios_from_user_story() -> String {
	r#"
You are a senior QA Engineer specializing in E2E testing.

From the User Story below, extract all possible Test Scenarios.

For each scenario, provide:
1. Scenario ID and descriptive name
2. Preconditions (initial state required before the test)
3. Steps (user actions in exact order)
4. Expected Results (what should be visible/happen after each key step)
5. Required Test Data
6. Category: Happy Path / Edge Case / Error Case / Boundary

Ensure full coverage across:
- Every happy path flow
- All validation errors
- Edge cases (empty input, max length, special characters, duplicates)
- Permission/authorization cases (if applicable)
- All possible state transitions

User Story:	
	"#.to_string()
}

pub fn extract_technical_context_from_code(code: String) -> String {
	format!(r#"
	You are a Frontend Developer preparing technical context
for a QA team to write Playwright E2E tests.
 
From the code below, extract the following:
 
1. **Page Routes/URLs**: every route path involved in this feature
2. **Selectors Map**: every element the user can interact with
   - Use Playwright-preferred selector priority:
    selector ID> role + name > label > CSS selector
   - Format: {{ elementName, selector ID, action , action ID: click/fill/select/etc }}
   - Format constraint : get and provide all ID that has respond
4. **UI States & Conditions**: conditions that change the UI
   (loading, error, empty, success, disabled states)
5. **Form Validations**: all validation rules
   (required, min/max, pattern, custom validators)
6. **Navigation Flow**: any redirects or page transitions triggered
   by user actions
 
Code:
{}
	"#, code).to_string()
}