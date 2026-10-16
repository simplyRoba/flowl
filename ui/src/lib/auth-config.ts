export interface EnabledAuthConfig {
  enabled: true;
  provider_name: string;
}

export interface DisabledAuthConfig {
  enabled: false;
}

export type AuthConfig = EnabledAuthConfig | DisabledAuthConfig;

/** Decodes only the public, normalized authentication configuration contract. */
export function parseAuthConfig(value: unknown): AuthConfig | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return null;
  }
  const keys = Object.keys(value);
  if (!keys.includes("enabled") || !("enabled" in value)) return null;
  if (value.enabled === false && keys.length === 1) {
    return { enabled: false };
  }
  if (
    value.enabled !== true ||
    keys.length !== 2 ||
    !keys.includes("provider_name") ||
    !("provider_name" in value) ||
    typeof value.provider_name !== "string"
  ) {
    return null;
  }
  const name = value.provider_name;
  if (
    name.length === 0 ||
    name.trim() !== name ||
    [...name].length > 80 ||
    [...name].some((character) => {
      const code = character.codePointAt(0) ?? 0;
      return code <= 0x1f || (code >= 0x7f && code <= 0x9f);
    })
  ) {
    return null;
  }
  return { enabled: true, provider_name: name };
}
