// Worker-safe compatibility exports: this module has no app/navigation side effects.
export { parseAuthConfig } from "./auth-core.js";
export type {
  AuthConfig,
  EnabledAuthConfig,
  DisabledAuthConfig,
} from "./auth-core.js";
