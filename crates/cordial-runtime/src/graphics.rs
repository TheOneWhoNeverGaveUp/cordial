//! Which graphics backend the engine is given, and who asked for it.
//!
//! **In this fork the answer is always GLES3.** The virtual `libvulkan.so` and
//! `libvulkan.so.1` sonames are never registered (see `symtab::build`), so the
//! engine's `dlopen` of them fails and it takes its own documented
//! fall-through to GLES3. There is no Vulkan to offer and no setting that
//! changes that.
//!
//! The `vulkan` spelling still parses — the shell's Renderer row still offers
//! it — and selecting it fires the sentinel rather than quietly doing nothing,
//! because a setting that looks set and silently does nothing is the failure
//! this module exists because of. `automatic` is the default and means GLES3,
//! full stop: the "if available" it used to carry is a branch that no longer
//! exists.
//!
//! ## Precedence, which is the user's and then the plugins'
//!
//! The same rule [`crate::flags`] already applies to everything else: an
//! explicit setting is the one thing that must not be quietly overridden, and a
//! plugin gets its say only where the user has not had one. `Automatic` is not
//! a third opinion competing with those two — it is the absence of a user
//! opinion, which is exactly what leaves the door open for a plugin.
//!
//! A plugin asks by writing [`KEY`] into its own flag layer. That key is
//! Cordial's rather than Roblox's, so it never reaches the engine's settings —
//! see `client_settings.rs`, which drops the `Cordial` prefix before applying.

use std::sync::OnceLock;

/// The Cordial-owned key a plugin writes to ask for a backend.
///
/// Deliberately not a Roblox flag name. It rides the flag layering because that
/// machinery already carries precedence and provenance, not because the engine
/// has any idea what it means.
pub const KEY: &str = "CordialGraphicsBackend";

/// The environment variable the shell sets from the Graphics row.
///
/// Set only when the user has chosen something other than Automatic: an absent
/// variable and `automatic` mean the same thing, and the shell sends the
/// variable rather than a file because the backend has to be known before the
/// first `dlopen`, which is well before anything reads a profile.
pub const ENV: &str = "CORDIAL_GRAPHICS";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// The absence of a user opinion, which is what leaves the door open for a
    /// plugin. Means GLES3 in this fork: the "Vulkan if available" it used to
    /// carry is a branch that no longer exists.
    Automatic,
    /// Ask for Vulkan, which is disconnected in this fork. The spelling still
    /// parses so the failure is loud — selecting it fires the sentinel — rather
    /// than a setting that looks set and silently does nothing.
    Vulkan,
    /// GLES3, explicitly. The only backend there is.
    GlEs,
}

impl Backend {
    pub fn parse(text: &str) -> Option<Backend> {
        match text.trim().to_ascii_lowercase().as_str() {
            "automatic" | "auto" | "" => Some(Backend::Automatic),
            "vulkan" => Some(Backend::Vulkan),
            "gles" | "gles3" | "glsles3" | "opengl" | "opengles" => Some(Backend::GlEs),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Backend::Automatic => "automatic",
            Backend::Vulkan => "vulkan",
            Backend::GlEs => "gles",
        }
    }

    /// Whether Cordial should offer the engine a Vulkan loader.
    ///
    /// It never does in this fork, and the question is no longer a `bool`:
    /// `GlEs` withholds Vulkan (and always did), while `Automatic` and
    /// `Vulkan` reach for a backend that is disconnected, which fires the
    /// sentinel. Nothing in the running client calls this — the loader it
    /// used to gate is gone — so it survives as the greppable crash site for
    /// a path that reaches for Vulkan.
    pub fn offers_vulkan(self) -> bool {
        match self {
            Backend::GlEs => false,
            Backend::Automatic | Backend::Vulkan => crate::android::vulkan::vulkan_sentinel(),
        }
    }
}

/// The backend in force, and the words for who asked.
#[derive(Debug, Clone)]
pub struct Choice {
    pub backend: Backend,
    /// Human-readable provenance — `"the Graphics setting"`, `"plugin:foo"`.
    ///
    /// Carried rather than derived because the whole point is that a plugin
    /// silently changing somebody's renderer must be diagnosable: "my game got
    /// slow after installing a plugin" should be one line in a log, not an
    /// afternoon.
    pub source: String,
}

/// Resolve once, per process.
pub fn choice() -> &'static Choice {
    static CHOICE: OnceLock<Choice> = OnceLock::new();
    CHOICE.get_or_init(|| resolve(std::env::var(ENV).ok(), plugin_request()))
}

/// The decision itself, with both inputs passed in so it can be tested.
pub fn resolve(from_env: Option<String>, from_plugin: Option<(String, String)>) -> Choice {
    // The user first, and an unparseable value is reported rather than
    // silently treated as Automatic: a Graphics row that does nothing is the
    // failure this whole module exists because of.
    if let Some(text) = from_env.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
        match Backend::parse(text) {
            Some(Backend::Automatic) => {}
            Some(backend) => {
                return Choice { backend, source: "the Graphics setting".into() };
            }
            None => {
                eprintln!(
                    "[graphics] {ENV}={text:?} is not a backend; using Automatic. \
                     Known: automatic, vulkan, gles"
                );
                return Choice { backend: Backend::Automatic, source: "Automatic (after a bad {ENV})".into() };
            }
        }
    }

    // Then a plugin, which only gets here because the user said Automatic.
    if let Some((id, text)) = from_plugin {
        match Backend::parse(&text) {
            Some(Backend::Automatic) | None => {
                eprintln!(
                    "[graphics] {id} asked for {KEY}={text:?}, which is not a backend; ignoring"
                );
            }
            Some(backend) => return Choice { backend, source: id },
        }
    }

    Choice { backend: Backend::Automatic, source: "Automatic".into() }
}

/// What the flag layers say, if anything, and who said it.
fn plugin_request() -> Option<(String, String)> {
    let resolved = crate::flags::resolve(crate::flags::collect());
    let entry = resolved.get(KEY)?;
    Some((entry.source.describe(), entry.value.clone()))
}

/// Say which backend is in force and why, once, at startup.
///
/// Printed unconditionally rather than behind a trace switch. It is one line,
/// and the question it answers — "why is this slow" / "why does this look
/// different from yesterday" — is asked from a support thread where nobody is
/// going to be asked to reproduce with an environment variable set.
pub fn report() {
    let choice = choice();
    match choice.backend {
        Backend::Automatic | Backend::GlEs => println!(
            "[graphics] backend: GLES3 — Vulkan is not offered in this fork, from {}",
            choice.source
        ),
        Backend::Vulkan => crate::android::vulkan::vulkan_sentinel(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_spellings_that_are_accepted_are_the_ones_a_person_would_type() {
        for text in ["gles", "GLES", " GlEs3 ", "opengl", "glsles3"] {
            assert_eq!(Backend::parse(text), Some(Backend::GlEs), "{text}");
        }
        assert_eq!(Backend::parse("Vulkan"), Some(Backend::Vulkan));
        assert_eq!(Backend::parse(""), Some(Backend::Automatic));
        assert_eq!(Backend::parse("metal"), None);
    }

    #[test]
    fn the_user_beats_a_plugin_and_automatic_is_not_a_veto() {
        // The rule `flags.rs` states and this has to match: an explicit setting
        // wins, and Automatic is the absence of one rather than a third opinion.
        let plugin = || Some(("plugin:shiny".to_string(), "gles".to_string()));

        let explicit = resolve(Some("vulkan".into()), plugin());
        assert_eq!(explicit.backend, Backend::Vulkan);
        assert_eq!(explicit.source, "the Graphics setting");

        let deferred = resolve(Some("automatic".into()), plugin());
        assert_eq!(deferred.backend, Backend::GlEs);
        assert_eq!(deferred.source, "plugin:shiny", "Automatic must let a plugin through");

        let unset = resolve(None, plugin());
        assert_eq!(unset.backend, Backend::GlEs, "an absent variable is Automatic");
    }

    #[test]
    fn nothing_asking_means_automatic() {
        let none = resolve(None, None);
        assert_eq!(none.backend, Backend::Automatic);
        assert!(none.source.contains("Automatic"), "{}", none.source);
    }

    #[test]
    fn a_value_nobody_understands_falls_back_rather_than_guessing() {
        // Both directions, because the failure being avoided is the one the
        // FastFlag had: a value that is not understood must not look like it
        // worked. Automatic is the safe landing, and it is announced.
        let bad_env = resolve(Some("mantle".into()), None);
        assert_eq!(bad_env.backend, Backend::Automatic);

        let bad_plugin = resolve(None, Some(("plugin:x".into(), "mantle".into())));
        assert_eq!(bad_plugin.backend, Backend::Automatic);
        assert!(bad_plugin.source.contains("Automatic"), "{}", bad_plugin.source);
    }
}
