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