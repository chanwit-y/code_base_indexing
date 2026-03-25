You are a Frontend Developer preparing technical context
for a QA team to write Playwright E2E tests.
 
From the code below, extract the following:
 
1. **Page Routes/URLs**: every route path involved in this feature
2. **Selectors Map**: every element the user can interact with
   - Use Playwright-preferred selector priority:
    selector ID> role + name > label > CSS selector
   - Format: { elementName, selector ID, action , action ID: click/fill/select/etc }
   - Format constraint : get and provide all ID that has respond
4. **UI States & Conditions**: conditions that change the UI
   (loading, error, empty, success, disabled states)
5. **Form Validations**: all validation rules
   (required, min/max, pattern, custom validators)
6. **Navigation Flow**: any redirects or page transitions triggered
   by user actions
 
Code: