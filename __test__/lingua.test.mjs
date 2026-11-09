import assert from "node:assert/strict";
import { createRequire } from "node:module";
import test from "node:test";

const { LanguageDetector, supportedLanguages } = createRequire(import.meta.url)("../index.js");

test("lists every language the build supports", () => {
  const languages = supportedLanguages();
  assert.equal(languages.length, 75);
  assert.ok(languages.includes("en"));
  assert.deepEqual(languages, [...languages].sort());
});

test("detects one text", () => {
  const detector = new LanguageDetector(["en", "de"]);
  const result = detector.detect("Leitung Marketing");
  assert.equal(result.language, "de");
  assert.ok(result.confidence > 0.5 && result.confidence <= 1);
});

test("detects a batch in order", async () => {
  const detector = new LanguageDetector(["en", "es", "ca", "eu"], { minimumRelativeDistance: 0.1 });
  const results = await detector.detectMany(["Supervisor de operaciones", "Head of Sales", "Profesor de arte"]);
  assert.deepEqual(
    results.map((r) => r.language),
    ["es", "en", "es"]
  );
});

test("returns null when the top two languages are too close", () => {
  const detector = new LanguageDetector(["en", "it"], { minimumRelativeDistance: 0.1 });
  assert.deepEqual(detector.detect("Senior 3D Animator"), { language: null, confidence: 0 });
});

test("returns null for text with no letters", () => {
  const detector = new LanguageDetector(["en", "fr"]);
  assert.equal(detector.detect("1234 !!").language, null);
});

test("gives every language's confidence, highest first", () => {
  const detector = new LanguageDetector(["en", "fr", "de"]);
  const values = detector.confidenceValues("Responsable des ventes");
  assert.equal(values.length, 3);
  assert.equal(values[0].language, "fr");
  for (let i = 1; i < values.length; i++) assert.ok(values[i - 1].confidence >= values[i].confidence);
});

test("accepts codes in any case and ignores duplicates", () => {
  const detector = new LanguageDetector(["EN", "de", "de"]);
  assert.equal(detector.detect("Leitung Marketing").language, "de");
});

test("rejects bad input instead of crashing", () => {
  assert.throws(() => new LanguageDetector([]), /At least one language/);
  assert.throws(() => new LanguageDetector(["en", "xx", "yy"]), /xx, yy/);
  assert.throws(() => new LanguageDetector(["en", "de"], { minimumRelativeDistance: 1 }), /between 0 and 0.99/);
});
