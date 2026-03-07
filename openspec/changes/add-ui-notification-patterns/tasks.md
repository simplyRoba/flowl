## 0. Design artifacts

- [ ] 0.1 Maintain a self-contained HTML mockup covering desktop toast stack, mobile toast stack, field-inline, section-inline, and page-level scenarios

## 1. Shared notification foundation

- [ ] 1.1 Add a global notification store/model supporting `success`, `info`, `warning`, and `error`
- [ ] 1.2 Add a toast host component mounted from the app shell with responsive desktop/mobile placement
- [ ] 1.3 Add accessibility semantics, dismissal behavior, and tests for stacking and timeout rules

## 2. Apply the taxonomy to high-value flows

- [ ] 2.1 Update dashboard attention-card watering to show visible success/error feedback near the action time
- [ ] 2.2 Replace MQTT repair and import row-inline status text with global toast feedback
- [ ] 2.3 Add feedback for actions that navigate away or remove their own context (for example delete plant, delete location, save-note success)
- [ ] 2.4 Keep contextual inline flows inline (identify, chat stream errors, field validation)
- [ ] 2.5 Use the same toast failure pattern for plant create/update failures and photo-upload-during-save failures

## 3. Fill weak or silent error paths

- [ ] 3.1 Add or refine submission-failure feedback for retry-in-place flows, including care entry form behavior
- [ ] 3.2 Review export feedback and decide whether detectable failures should toast while success remains silent
- [ ] 3.3 Treat selected photo upload as part of save completion for create/edit flows, so navigation does not complete when the photo step fails
- [ ] 3.4 Define the retry/recovery UX for partial success when plant create/update succeeds but photo upload fails

## 4. Verification

- [ ] 4.1 Add or update UI tests for the notification host and the first integrated flows
- [ ] 4.2 Run `npm run check --prefix ui`, `npm run lint --prefix ui`, and relevant UI tests
- [ ] 4.3 After implementation is complete, mark related `REVIEW.md` items resolved, including item 10 (dashboard watering feedback) and item 50 (global toast/snackbar system)
