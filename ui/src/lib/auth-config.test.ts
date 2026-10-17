import { describe, expect, it } from "vitest";

import { parseAuthConfig } from "./auth-config.js";

describe("public auth configuration decoder", () => {
  it("accepts only the exact disabled shape", () => {
    expect(parseAuthConfig({ enabled: false })).toEqual({ enabled: false });
  });

  it.each([
    "OpenID Connect",
    '<provider>&"name',
    "é".repeat(80),
    "🌱".repeat(80),
  ])("accepts normalized label %s", (provider_name) => {
    const config = { enabled: true, provider_name, auto_redirect: false };
    expect(parseAuthConfig(config)).toEqual(config);
  });

  it.each([
    null,
    undefined,
    [],
    [false],
    "false",
    false,
    0,
    {},
    { enabled: "false" },
    { enabled: null },
    { enabled: false, provider_name: null },
    { enabled: false, provider_name: "SSO" },
    { enabled: false, extra: true },
    { enabled: true },
    { enabled: true, provider_name: null, auto_redirect: false },
    { enabled: true, provider_name: 42, auto_redirect: false },
    { enabled: true, provider_name: "SSO", auto_redirect: false, extra: true },
    { enabled: true, provider_name: "SSO" },
    { enabled: true, provider_name: "SSO", auto_redirect: "false" },
    ...[
      "",
      " ",
      " SSO",
      "SSO ",
      "a".repeat(81),
      "é".repeat(81),
      "🌱".repeat(81),
      "bad\nname",
      "bad\0name",
      "bad\u007fname",
      "bad\u0085name",
      "bad\u009fname",
    ].map((provider_name) => ({
      enabled: true,
      provider_name,
      auto_redirect: false,
    })),
  ])("rejects malformed configuration %j", (value) => {
    expect(parseAuthConfig(value)).toBeNull();
  });
});
