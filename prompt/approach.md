# High-Level Approach to Network Troubleshooting

## Methodology for Investigation

Follow these steps when diagnosing issues:

1. **Review Existing Analyses:** Consult the documents in `docs/*-analysis.md` to understand the current state and previous findings.
2. **Understand Custom Utilities:** Read the README files for the Rust-based utilities developed for this project. These tools encapsulate domain-specific logic.
3. **Consult Vendor Documentation:** Review relevant sections of the manuals located in `docs/*-manual/all.md`.
4. **Prioritize Custom Tools:** Prefer the provided Rust utilities over generic command-line networking tools (e.g., `tcpdump`, `curl`) when possible.
5. **Extend Tools as Needed:** If a lower-level command-line tool is required, consider extending the corresponding Rust utility with the missing functionality. Validate that the new Rust implementation meets your requirements.
6. **Document Findings:** Continuously update the analysis documents as you uncover new information.

## Methodology for Tool Development

Follow these steps when extending the Rust-based configuration and diagnostic tools:

1. **Review Existing Analyses:** Consult the documents in `docs/*-analysis.md` for context.
2. **Consult Vendor Documentation:** Read relevant sections of the hardware manuals in `docs/*-manual/all.md`.
3. **Implement Feature Enhancements:** Add the missing functionality by expanding on the existing codebase. Avoid replacing existing, working functionality unless necessary.
4. **Add Unit Tests:** Implement comprehensive unit tests to verify the behavior and document your understanding of the system.
5. **Validate Against Hardware:** Run the modified tool against the actual networking equipment to ensure it behaves as intended in a live environment.
6. **Update Documentation:** Revise the tool's README, inline comments, and command-line help text to reflect the new functionality.
7. **Synchronize Analysis Documents:** Update the analysis documentation if your findings contradict or expand upon the existing information.
