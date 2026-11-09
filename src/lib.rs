#![deny(clippy::all)]

//! Node.js bindings for lingua (https://github.com/pemistahl/lingua-rs).
//!
//! Languages are named by their ISO 639-1 code (`"en"`, `"de"`), in and out.
//! Every input is checked before it reaches lingua, because lingua panics on
//! an empty language list or an out-of-range distance, and a panic would take
//! the Node process down with it.

use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

use lingua::{IsoCode639_1, Language, LanguageDetectorBuilder};
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Options for building a {@link LanguageDetector}.
#[napi(object)]
pub struct DetectorOptions {
  /// How far apart, from 0 to 0.99, the top two candidates' confidences must
  /// be before a language is named. Below it, `detect` returns null rather
  /// than a guess. Defaults to 0.
  pub minimum_relative_distance: Option<f64>,
  /// Trade accuracy on short text for speed. Defaults to false.
  pub low_accuracy_mode: Option<bool>,
  /// Load every model when the detector is built instead of on first use.
  /// Defaults to false.
  pub preload_language_models: Option<bool>,
}

/// The language one text was found to be written in.
#[napi(object)]
pub struct Detection {
  /// ISO 639-1 code, or null when no language could be named.
  #[napi(ts_type = "string | null")]
  pub language: Either<String, Null>,
  /// lingua's confidence in `language`, from 0 to 1, relative to the other
  /// languages the detector was built with. 0 when `language` is null.
  pub confidence: f64,
}

/// One language's confidence for a text.
#[napi(object)]
pub struct LanguageConfidence {
  /// ISO 639-1 code.
  pub language: String,
  /// From 0 to 1.
  pub confidence: f64,
}

/// The ISO 639-1 codes of every language this build can detect, sorted.
#[napi]
pub fn supported_languages() -> Vec<String> {
  let mut codes: Vec<String> = Language::all().iter().map(code).collect();
  codes.sort();
  codes
}

/// Detects which of a fixed set of languages a text is written in.
#[napi]
pub struct LanguageDetector {
  inner: Arc<lingua::LanguageDetector>,
}

#[napi]
impl LanguageDetector {
  /// Builds a detector that chooses between `languages`, given as ISO 639-1
  /// codes. Fewer languages make for more accurate results on short text.
  #[napi(constructor)]
  pub fn new(languages: Vec<String>, options: Option<DetectorOptions>) -> Result<Self> {
    let iso_codes = parse_languages(&languages)?;
    let mut builder = LanguageDetectorBuilder::from_iso_codes_639_1(&iso_codes);

    if let Some(options) = options {
      if let Some(distance) = options.minimum_relative_distance {
        if !(0.0..=0.99).contains(&distance) {
          return Err(invalid(format!(
            "minimumRelativeDistance must be between 0 and 0.99, not {distance}"
          )));
        }
        builder.with_minimum_relative_distance(distance);
      }
      if options.low_accuracy_mode == Some(true) {
        builder.with_low_accuracy_mode();
      }
      if options.preload_language_models == Some(true) {
        builder.with_preloaded_language_models();
      }
    }

    Ok(Self {
      inner: Arc::new(builder.build()),
    })
  }

  /// The language of one text. Runs on the calling thread.
  #[napi]
  pub fn detect(&self, text: String) -> Detection {
    match self.inner.detect_language_of(text.as_str()) {
      Some(language) => Detection {
        language: Either::A(code(&language)),
        confidence: self.inner.compute_language_confidence(text, language),
      },
      None => Detection {
        language: Either::B(Null),
        confidence: 0.0,
      },
    }
  }

  /// The language of each text, in the order given. Runs off the main thread,
  /// spreading the batch across the CPU's cores.
  #[napi(ts_return_type = "Promise<Array<Detection>>")]
  pub fn detect_many(&self, texts: Vec<String>) -> AsyncTask<DetectMany> {
    AsyncTask::new(DetectMany {
      detector: Arc::clone(&self.inner),
      texts,
    })
  }

  /// Every language's confidence for one text, highest first.
  #[napi]
  pub fn confidence_values(&self, text: String) -> Vec<LanguageConfidence> {
    self
      .inner
      .compute_language_confidence_values(text)
      .into_iter()
      .map(|(language, confidence)| LanguageConfidence {
        language: code(&language),
        confidence,
      })
      .collect()
  }
}

/// The work behind `detectMany`, run on libuv's thread pool.
pub struct DetectMany {
  detector: Arc<lingua::LanguageDetector>,
  texts: Vec<String>,
}

impl Task for DetectMany {
  type Output = Vec<Detection>;
  type JsValue = Vec<Detection>;

  fn compute(&mut self) -> Result<Self::Output> {
    let texts = std::mem::take(&mut self.texts);
    let languages = self.detector.detect_languages_in_parallel_of(&texts);
    let confidences = self
      .detector
      .compute_language_confidence_values_in_parallel(&texts);

    Ok(
      languages
        .into_iter()
        .zip(confidences)
        .map(|(language, values)| match language {
          Some(language) => Detection {
            language: Either::A(code(&language)),
            confidence: values
              .iter()
              .find(|(candidate, _)| *candidate == language)
              .map_or(0.0, |(_, confidence)| *confidence),
          },
          None => Detection {
            language: Either::B(Null),
            confidence: 0.0,
          },
        })
        .collect(),
    )
  }

  fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
    Ok(output)
  }
}

fn code(language: &Language) -> String {
  language.iso_code_639_1().to_string()
}

fn invalid(message: String) -> Error {
  Error::new(Status::InvalidArg, message)
}

/// The languages as lingua's ISO codes, or an error naming every code it does
/// not support.
fn parse_languages(languages: &[String]) -> Result<Vec<IsoCode639_1>> {
  if languages.is_empty() {
    return Err(invalid("At least one language is required".to_string()));
  }

  let mut codes = Vec::with_capacity(languages.len());
  let mut seen = HashSet::new();
  let mut unknown = Vec::new();
  for language in languages {
    match IsoCode639_1::from_str(language.trim()) {
      Ok(iso_code) => {
        if seen.insert(iso_code) {
          codes.push(iso_code);
        }
      }
      Err(_) => unknown.push(language.as_str()),
    }
  }

  if !unknown.is_empty() {
    return Err(invalid(format!(
      "lingua does not support these ISO 639-1 codes: {}",
      unknown.join(", ")
    )));
  }
  Ok(codes)
}
