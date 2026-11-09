# @prospectbase/lingua: language detection for Node.js

[![npm](https://img.shields.io/npm/v/@prospectbase/lingua)](https://www.npmjs.com/package/@prospectbase/lingua)
[![CI](https://github.com/prospectbase/lingua/actions/workflows/CI.yml/badge.svg)](https://github.com/prospectbase/lingua/actions/workflows/CI.yml)
[![licence](https://img.shields.io/badge/licence-Apache--2.0-blue)](LICENSE)

**Detect the language of a string in Node.js and TypeScript**, accurately even on short text: a single word, a name, a job title, a tweet, a search query. It identifies **75 languages** and runs offline, with no API key, network call or model download.

This package is native Node.js bindings for [lingua](https://github.com/pemistahl/lingua-rs), the Rust language detection library known for its accuracy on short text. It's compiled with [napi-rs](https://napi.rs) and ships prebuilt binaries for Windows, macOS and Linux, so there's nothing to compile when you install.

```ts
import { LanguageDetector } from "@prospectbase/lingua";

const detector = new LanguageDetector(["en", "de", "fr", "es"]);
detector.detect("Leitung Marketing"); // { language: "de", confidence: 0.77 }
```

## Why this package

- **Accurate on short text.** Most language detectors count character n-grams and need a sentence or more to be reliable. lingua combines a rule-based step with statistical models trained on up to 5-grams, and lets you limit it to the languages you expect. It does well on input of one to three words, where n-gram detectors tend to guess. lingua's README publishes [accuracy comparisons](https://github.com/pemistahl/lingua-rs#4-how-accurate-is-it) with CLD2, Whatlang and Whichlang.
- **Native, not WebAssembly, so it's light on memory.** The language models are compiled into the binary, and the operating system loads only the parts a detector reads. A Node.js process running two detectors peaks at about 90 MB, Node's own ~50 MB included. A WebAssembly build of the same library holds every model in memory at once, about 900 MB. That fits a small server, container or background worker.
- **Fast batches off the main thread.** `detectMany` runs on libuv's thread pool and spreads a batch across the CPU's cores, so a large batch doesn't block the event loop.
- **Confidence scores, and no guessing.** Every result carries a confidence from 0 to 1. With `minimumRelativeDistance` set, the detector returns `null` instead of guessing when two languages are too close to call.
- **Typed.** It ships TypeScript definitions, and works from CommonJS and ES modules.
- **Safe input checking.** An unknown language code or a bad option throws a JavaScript error rather than crashing the process.

The trade-off is speed: lingua is slower per text than n-gram detectors like CLD2. Choose it when accuracy on short text matters more than raw throughput.

## Install

```bash
npm install @prospectbase/lingua
```

npm installs only the binary for your platform, as an optional dependency. It's prebuilt for:

| OS | Architectures |
|---|---|
| macOS | arm64 (Apple Silicon), x64 (Intel) |
| Linux, glibc (Debian, Ubuntu, RHEL, Amazon Linux, …) | x64, arm64 |
| Windows | x64 |

It needs Node.js 20 or later. The Windows binary links its C runtime statically, so it doesn't need the Visual C++ Redistributable. Alpine (musl) and Windows on Arm aren't built yet.

## How to detect the language of text in Node.js

```ts
import { LanguageDetector, supportedLanguages } from "@prospectbase/lingua";

// Choose between as few languages as you can: lingua is more accurate on short
// text when it has fewer candidates.
const detector = new LanguageDetector(["en", "es", "ca"], { minimumRelativeDistance: 0.1 });

// One text, on the calling thread.
detector.detect("Supervisor de operaciones");
// { language: "es", confidence: 0.77 }

// A batch, off the main thread, results in the same order.
await detector.detectMany(["Head of Sales", "Senior 3D Animator"]);
// [{ language: "en", confidence: 0.59 }, { language: "en", confidence: 0.64 }]

// Every candidate's confidence, highest first.
detector.confidenceValues("Profesor de arte");
// [{ language: "es", confidence: 0.75 }, { language: "ca", confidence: 0.16 }, { language: "en", confidence: 0.08 }]

// When the top two languages are closer than minimumRelativeDistance, no
// language is named.
new LanguageDetector(["en", "it"], { minimumRelativeDistance: 0.1 }).detect("Senior 3D Animator");
// { language: null, confidence: 0 }

supportedLanguages(); // ["af", "ar", "az", ...]
```

From CommonJS:

```js
const { LanguageDetector } = require("@prospectbase/lingua");
```

## API

### `new LanguageDetector(languages, options?)`

`languages` is an array of [ISO 639-1](https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes) codes, in any case. An empty array, an unknown code or an out-of-range option throws an error that names the problem.

| Option | Default | |
|---|---|---|
| `minimumRelativeDistance` | `0` | From 0 to 0.99. How far apart the top two candidates' confidences must be before a language is named. Below it, the result is `null` rather than a guess. 0.1 is a good start for short text. |
| `lowAccuracyMode` | `false` | Uses only the trigram models. It's faster and uses less memory, but is less accurate on short text. |
| `preloadLanguageModels` | `false` | Loads every model when the detector is built instead of on first use. |

### `detector.detect(text): Detection`

`Detection` is `{ language: string | null, confidence: number }`.
- `language` is a code, or `null` when no language could be named: the text has no letters, or the top two candidates are closer than `minimumRelativeDistance`.
- `confidence` is from 0 to 1, relative to the detector's languages, and is 0 when `language` is `null`.

### `detector.detectMany(texts): Promise<Detection[]>`

The same, for a batch, in input order. It runs on libuv's thread pool across the CPU's cores. Use it for anything more than a handful of texts.

### `detector.confidenceValues(text): LanguageConfidence[]`

Every language the detector was built with, as `{ language, confidence }`, highest confidence first.

### `supportedLanguages(): string[]`

The ISO 639-1 codes of every language this build can detect, sorted.

## Supported languages

All 75 languages lingua supports are in every build:

| Language | Code | Language | Code | Language | Code |
|---|---|---|---|---|---|
| Afrikaans | `af` | Gujarati | `gu` | Romanian | `ro` |
| Albanian | `sq` | Hebrew | `he` | Russian | `ru` |
| Arabic | `ar` | Hindi | `hi` | Serbian | `sr` |
| Armenian | `hy` | Hungarian | `hu` | Shona | `sn` |
| Azerbaijani | `az` | Icelandic | `is` | Slovak | `sk` |
| Basque | `eu` | Indonesian | `id` | Slovene | `sl` |
| Belarusian | `be` | Irish | `ga` | Somali | `so` |
| Bengali | `bn` | Italian | `it` | Sotho | `st` |
| Bosnian | `bs` | Japanese | `ja` | Spanish | `es` |
| Bulgarian | `bg` | Kazakh | `kk` | Swahili | `sw` |
| Catalan | `ca` | Korean | `ko` | Swedish | `sv` |
| Chinese | `zh` | Latin | `la` | Tagalog | `tl` |
| Croatian | `hr` | Latvian | `lv` | Tamil | `ta` |
| Czech | `cs` | Lithuanian | `lt` | Telugu | `te` |
| Danish | `da` | Macedonian | `mk` | Thai | `th` |
| Dutch | `nl` | Malay | `ms` | Tsonga | `ts` |
| English | `en` | Maori | `mi` | Tswana | `tn` |
| Esperanto | `eo` | Marathi | `mr` | Turkish | `tr` |
| Estonian | `et` | Mongolian | `mn` | Ukrainian | `uk` |
| Finnish | `fi` | Norwegian Bokmål | `nb` | Urdu | `ur` |
| French | `fr` | Norwegian Nynorsk | `nn` | Vietnamese | `vi` |
| Ganda | `lg` | Persian | `fa` | Welsh | `cy` |
| Georgian | `ka` | Polish | `pl` | Xhosa | `xh` |
| German | `de` | Portuguese | `pt` | Yoruba | `yo` |
| Greek | `el` | Punjabi | `pa` | Zulu | `zu` |

## Common uses

- **Routing text to translation:** find which titles, product names or messages aren't in English before paying for a translation.
- **Language detection for search and NLP pipelines:** choose a stemmer, tokenizer or spell checker per document.
- **Data cleaning:** tag or filter records by language in CRM, e-commerce or survey data.
- **Content moderation and support routing:** send messages to the right team or model by language.

## Develop

You need Rust (stable) and Node.js 22 or later.

```bash
npm ci
npm run build   # release build for this machine: lingua.<platform>.node, index.js, index.d.ts
npm test
```

`index.js` (the loader that picks the platform's binary) and `index.d.ts` are generated by `napi build` from `src/lib.rs`. Commit them whenever the binding's API changes.

A release build compiles every language model in and takes under a minute on an Apple Silicon Mac. The binary is about 290 MB.

## Release

CI (`.github/workflows/CI.yml`) builds and tests every target on each push to `main`, and tests the Linux binaries inside Debian bookworm. It publishes when the head commit is a version commit:

```bash
npm version patch   # or minor / major; commits "x.y.z"
git push
```

Publishing reads the npm token in the repository secret `NPM_TOKEN`. `npm publish` runs `napi prepublish`, which publishes one package per platform (`@prospectbase/lingua-darwin-arm64`, `-linux-x64-gnu`, …) and points the main package's `optionalDependencies` at them.

### Upgrading lingua

Bump the version pinned in `Cargo.toml` (`lingua = "=x.y.z"`), rebuild, run the tests, and release. Read lingua's release notes first: a release can add languages, which changes `supportedLanguages()`, the table above and the test that counts them.

## Credits and licence

The language detection is entirely [lingua](https://github.com/pemistahl/lingua-rs), by Peter M. Stahl. This package adds the Node.js binding. Both are licensed Apache-2.0: see `LICENSE` and `NOTICE`.
