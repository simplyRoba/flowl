import { describe, expect, it } from "vitest";
import { de } from "./de";
import { en } from "./en";
import { es } from "./es";
import { plural } from "./plural";

describe("watering group date ranges", () => {
  it.each([
    [
      "English",
      en,
      "Watered 3 times, Feb 1 – Feb 3",
      "Watered 3+ times, Feb 1 – Feb 3",
    ],
    ["German", de, "3× gegossen, Feb 1 – Feb 3", "3+× gegossen, Feb 1 – Feb 3"],
    [
      "Spanish",
      es,
      "Regada 3 veces, Feb 1 – Feb 3",
      "Regada 3+ veces, Feb 1 – Feb 3",
    ],
  ])(
    "renders complete and partial ranges in %s",
    (_locale, translations, exact, partial) => {
      const replacements = (template: string) =>
        template
          .replace("{count}", "3")
          .replace("{from}", "Feb 1")
          .replace("{to}", "Feb 3");

      expect(replacements(translations.care.wateredSince)).toBe(exact);
      expect(replacements(translations.care.partialWateredSince)).toBe(partial);
    },
  );
});

describe("care-event error translations", () => {
  it.each([
    ["English", en],
    ["German", de],
    ["Spanish", es],
  ])("includes required care-event errors in %s", (_locale, translations) => {
    expect(translations.errorCode.CARE_EVENT_NOTES_REQUIRED).toBeTruthy();
    expect(translations.errorCode.CARE_EVENT_OCCURRED_AT_REQUIRED).toBeTruthy();
    expect(translations.errorCode.CARE_EVENT_INVALID_OCCURRED_AT).toBeTruthy();
  });
});

describe("plural", () => {
  it("returns the singular form for one", () => {
    expect(plural({ one: "{n} plant", other: "{n} plants" }, 1)).toBe(
      "1 plant",
    );
  });

  it("returns the plural form for many", () => {
    expect(plural({ one: "{n} plant", other: "{n} plants" }, 5)).toBe(
      "5 plants",
    );
  });

  it("uses the plural form for zero", () => {
    expect(plural({ one: "{n} plant", other: "{n} plants" }, 0)).toBe(
      "0 plants",
    );
  });
});
