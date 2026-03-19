## Types: Rust → TypeScript (auto-generated)

Types in `frontend/src/types/` are generated from Rust structs — **never edit manually**.

### Workflow

1. Add `#[ts(export)]` to the struct in `backend/`
2. Run `cd backend && cargo test export_bindings`
3. Commit Rust + generated TypeScript together

## Type checking for frontend

Always use `npx tsc -b --noEmit` (not `npx tsc --noEmit`)
