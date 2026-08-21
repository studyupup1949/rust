use advisor_core::{
    CatalogEntry, Confidence, GoalFit, GoalFitStrength, RecommendationArchetype, Tradeoff,
    TrustNote,
};

pub fn seed_catalog() -> Vec<CatalogEntry> {
    vec![
        entry(
            "clap",
            "cli-parsing",
            "Best default for feature-rich application CLIs and Cargo-style subcommands.",
            RecommendationArchetype::BestDefault,
            &[
                "Mature derive and builder APIs cover most CLI ergonomics without extra assembly.",
                "Fits teams that want shell completion, help text, and subcommand structure in one crate.",
            ],
            &[
                fit(
                    "boring-default",
                    GoalFitStrength::Strong,
                    "It is the safest checked-in default when you want the mainstream CLI path.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Strong,
                    "Derive-heavy workflows keep full-featured CLIs moving quickly.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "compile time",
                "The convenience surface is larger than leaner argument parsers.",
            )],
        ),
        entry(
            "argh",
            "cli-parsing",
            "Lean option for small CLIs that value a smaller parser surface.",
            RecommendationArchetype::LeanOption,
            &[
                "Keeps the API direct for straightforward flags and options.",
                "Useful when binary size and dependency restraint matter more than advanced CLI features.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Strong,
                    "It stays lighter than the more feature-complete CLI stacks in this catalog.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "Small CLIs can ship quickly because the API surface stays narrow.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "feature breadth",
                "It offers fewer advanced CLI affordances than clap.",
            )],
        ),
        entry(
            "bpaf",
            "cli-parsing",
            "Power option when you want composable parsers and more explicit control.",
            RecommendationArchetype::PowerOption,
            &[
                "Combinator style keeps parsing decisions closer to code than derive-only flows.",
                "Useful when bespoke validation or parser composition matters.",
            ],
            &[
                fit(
                    "maximum-control",
                    GoalFitStrength::Strong,
                    "The parser-combinator style gives more control than derive-first wrappers.",
                ),
                fit(
                    "typed-surfaces",
                    GoalFitStrength::Good,
                    "The explicit parser model rewards teams that want types and behavior wired together deliberately.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "learning curve",
                "The style is less familiar than the default derive-first path.",
            )],
        ),
        entry(
            "lexopt",
            "cli-parsing",
            "Specialist pick for hand-rolled CLIs that want very little abstraction.",
            RecommendationArchetype::Specialist,
            &[
                "Useful when you want to own parsing flow directly without framework-style layers.",
                "Fits tiny CLIs that still need predictable low-level argument handling.",
            ],
            &[
                fit(
                    "maximum-control",
                    GoalFitStrength::Good,
                    "It keeps parsing decisions explicit and close to the call site.",
                ),
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Good,
                    "The abstraction layer stays small compared with all-in-one CLI crates.",
                ),
            ],
            Confidence::Low,
            &[tradeoff(
                "assembly",
                "You give up higher-level ergonomics and batteries-included CLI features.",
            )],
        ),
        entry(
            "config",
            "config",
            "Best default for layered configuration across files, environment variables, and overrides.",
            RecommendationArchetype::BestDefault,
            &[
                "Covers the common application pattern of merging several configuration sources.",
                "Good default when the application boundary is broader than a single env-only config struct.",
            ],
            &[
                fit(
                    "boring-default",
                    GoalFitStrength::Strong,
                    "It is the most conventional checked-in choice for app-level configuration layering.",
                ),
                fit(
                    "layered-config",
                    GoalFitStrength::Strong,
                    "Source layering is the main reason to start here.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "shape control",
                "Some teams prefer assembling providers more explicitly than config encourages.",
            )],
        ),
        entry(
            "confique",
            "config",
            "Lean option for typed config structs with less layering machinery.",
            RecommendationArchetype::LeanOption,
            &[
                "Keeps configuration close to application structs with less indirection.",
                "Works well when configuration sources stay simple and typed APIs matter more than source breadth.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Strong,
                    "It avoids some of the larger source-composition surface of heavier config stacks.",
                ),
                fit(
                    "typed-surfaces",
                    GoalFitStrength::Strong,
                    "Typed derive-driven configuration is the central value proposition.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "source breadth",
                "It is less compelling when you need many configuration providers or deep merge behavior.",
            )],
        ),
        entry(
            "figment",
            "config",
            "Power option when provider composition and explicit merges are part of the design.",
            RecommendationArchetype::PowerOption,
            &[
                "Provider composition is a strong fit when the team wants to reason about config inputs directly.",
                "Good when framework integrations or explicit merge steps matter.",
            ],
            &[
                fit(
                    "maximum-control",
                    GoalFitStrength::Strong,
                    "Provider composition gives more explicit control over how configuration is assembled.",
                ),
                fit(
                    "layered-config",
                    GoalFitStrength::Good,
                    "It still handles layered config well, but with more explicit wiring than config.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "ecosystem fit",
                "The value is highest in stacks that already like its provider model.",
            )],
        ),
        entry(
            "envy",
            "config",
            "Specialist option for env-only configuration surfaces.",
            RecommendationArchetype::Specialist,
            &[
                "Good when deployment conventions already push all configuration into environment variables.",
                "Useful for services that intentionally avoid file-backed configuration.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Good,
                    "Env-only configuration keeps the surface smaller than full layering frameworks.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "It can be quick to wire when environment variables are the whole story.",
                ),
            ],
            Confidence::Low,
            &[tradeoff(
                "scope",
                "It is not a general answer for multi-source configuration assembly.",
            )],
        ),
        entry(
            "tracing",
            "logging-tracing",
            "Best default for modern Rust applications that need structured telemetry.",
            RecommendationArchetype::BestDefault,
            &[
                "Spans and structured fields make it the strongest default for application observability.",
                "Ecosystem alignment lowers migration friction for instrumented services.",
            ],
            &[
                fit(
                    "boring-default",
                    GoalFitStrength::Strong,
                    "It is the ecosystem-default observability path for many new Rust services.",
                ),
                fit(
                    "rich-diagnostics",
                    GoalFitStrength::Strong,
                    "Structured events and spans are the main reason to choose it.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "migration",
                "Teams on plain log macros may need adapter or subscriber work.",
            )],
        ),
        entry(
            "log",
            "logging-tracing",
            "Lean option for libraries that only need a portable logging facade.",
            RecommendationArchetype::LeanOption,
            &[
                "Works well when a library wants logging hooks without choosing an application backend.",
                "Keeps the commitment small when structured telemetry is not the goal.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Strong,
                    "It stays smaller than structured telemetry stacks when a facade is enough.",
                ),
                fit(
                    "typed-surfaces",
                    GoalFitStrength::Weak,
                    "It does not help much when you specifically want structured observability data.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "observability depth",
                "It does not model spans or structured events like tracing.",
            )],
        ),
        entry(
            "slog",
            "logging-tracing",
            "Power option for teams already committed to a structured logging stack outside tracing.",
            RecommendationArchetype::PowerOption,
            &[
                "Provides structured logging patterns for teams that want to stay inside that ecosystem.",
                "Can fit established codebases that standardized on it before tracing became the default.",
            ],
            &[
                fit(
                    "maximum-control",
                    GoalFitStrength::Good,
                    "It gives explicit structured logging control for teams already bought into the model.",
                ),
                fit(
                    "rich-diagnostics",
                    GoalFitStrength::Good,
                    "Structured event output is still the point, even if the ecosystem default moved.",
                ),
            ],
            Confidence::Low,
            &[tradeoff(
                "ecosystem momentum",
                "Newer Rust projects more often align around tracing instead.",
            )],
        ),
        entry(
            "env_logger",
            "logging-tracing",
            "Specialist option for simple log-based applications that want low ceremony.",
            RecommendationArchetype::Specialist,
            &[
                "Useful for straightforward binaries already using the log facade.",
                "Good when the team wants a minimal setup and does not need tracing-style spans.",
            ],
            &[
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "A simple log facade plus env_logger is usually quick to enable.",
                ),
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Good,
                    "It stays lighter than tracing-based observability stacks when needs are basic.",
                ),
            ],
            Confidence::Low,
            &[tradeoff(
                "headroom",
                "Teams often outgrow it once they need structured or cross-cutting telemetry.",
            )],
        ),
        entry(
            "reqwest",
            "http-client",
            "Best default for application HTTP clients with async and blocking support.",
            RecommendationArchetype::BestDefault,
            &[
                "Broad ecosystem familiarity lowers adoption risk.",
                "Covers most app-level HTTP client needs without much assembly.",
            ],
            &[
                fit(
                    "boring-default",
                    GoalFitStrength::Strong,
                    "It is the safest checked-in general-purpose client choice.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Strong,
                    "High-level ergonomics reduce the amount of HTTP plumbing teams need to write.",
                ),
                fit(
                    "blocking",
                    GoalFitStrength::Good,
                    "It can still satisfy blocking use cases without forcing a separate client choice.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Good,
                    "Async usage is first-class in mainstream application stacks.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "footprint",
                "The convenience surface can pull in more dependencies than smaller clients.",
            )],
        ),
        entry(
            "ureq",
            "http-client",
            "Lean option for simple blocking-first HTTP flows.",
            RecommendationArchetype::LeanOption,
            &[
                "Small and direct for straightforward request-response work.",
                "Good when async is unnecessary and the team wants a lighter stack.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Strong,
                    "It is lighter than the general-purpose application clients in this catalog.",
                ),
                fit(
                    "blocking",
                    GoalFitStrength::Strong,
                    "Blocking-first use cases are where it fits best.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Weak,
                    "It is not the right answer when async-first integration is the primary goal.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "async support",
                "It is not the default choice for async-heavy applications.",
            )],
        ),
        entry(
            "hyper",
            "http-client",
            "Power option when you need lower-level HTTP transport control.",
            RecommendationArchetype::PowerOption,
            &[
                "Useful as a foundation when higher-level clients are too opinionated.",
                "Fits teams comfortable composing the rest of the client stack themselves.",
            ],
            &[
                fit(
                    "maximum-control",
                    GoalFitStrength::Strong,
                    "It gives more control over the transport layer than the higher-level client defaults.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Good,
                    "It is at home in async-heavy networking stacks.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "ergonomics",
                "App-level client behavior usually takes more assembly work than reqwest.",
            )],
        ),
        entry(
            "isahc",
            "http-client",
            "Specialist option for teams that want a different high-level client posture than reqwest.",
            RecommendationArchetype::Specialist,
            &[
                "Provides another high-level HTTP client direction when reqwest is not the preferred fit.",
                "Useful when the team wants a checked-in alternative rather than jumping straight to hyper.",
            ],
            &[
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "It still targets application-level HTTP ergonomics rather than transport assembly.",
                ),
                fit(
                    "blocking",
                    GoalFitStrength::Good,
                    "It remains a plausible path for non-async application HTTP work.",
                ),
            ],
            Confidence::Low,
            &[tradeoff(
                "default gravity",
                "It is a more niche choice than reqwest for mainstream Rust application clients.",
            )],
        ),
        entry(
            "axum",
            "http-server",
            "Best default for modern Tokio-based HTTP APIs with approachable handler ergonomics.",
            RecommendationArchetype::BestDefault,
            &[
                "Router and extractor ergonomics make it approachable for service teams.",
                "Strong alignment with tracing and tower-based middleware keeps the stack coherent.",
            ],
            &[
                fit(
                    "boring-default",
                    GoalFitStrength::Strong,
                    "It is the safest modern application-server default in this catalog.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Strong,
                    "It fits async-first services without extra translation layers.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "The ergonomics reduce handler boilerplate for typical APIs.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "stack assumptions",
                "The best experience assumes comfort with Tokio and tower patterns.",
            )],
        ),
        entry(
            "poem",
            "http-server",
            "Lean option for teams that want a lighter-feeling async web framework.",
            RecommendationArchetype::LeanOption,
            &[
                "Useful when the team wants API ergonomics without the heaviest framework surface.",
                "Good fit for straightforward service layers that still want an async-native server.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Good,
                    "It is a lighter-feeling option than some of the larger server frameworks.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "Handler ergonomics can keep straightforward API work moving quickly.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "ecosystem gravity",
                "The default ecosystem center of gravity is still stronger around axum.",
            )],
        ),
        entry(
            "actix-web",
            "http-server",
            "Power option when throughput tuning and an established Actix model are already part of the standard.",
            RecommendationArchetype::PowerOption,
            &[
                "Mature server framework with a long track record.",
                "Can be a strong fit for teams that already know its execution model well.",
            ],
            &[
                fit(
                    "maximum-control",
                    GoalFitStrength::Good,
                    "It appeals most when the team wants a more deliberate server framework model than the default.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Good,
                    "It still serves async-heavy services well when the team is comfortable with the stack.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "ecosystem style",
                "Its style is less aligned with tower-centric stacks than axum.",
            )],
        ),
        entry(
            "warp",
            "http-server",
            "Specialist option for filter-composition-heavy routing styles.",
            RecommendationArchetype::Specialist,
            &[
                "Good for teams that actively prefer filter composition.",
                "Can express compact routing when the style matches the team's tastes.",
            ],
            &[
                fit(
                    "maximum-control",
                    GoalFitStrength::Good,
                    "Filter composition offers precise control when the team likes the model.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Weak,
                    "The routing style can become harder to move quickly with as APIs grow.",
                ),
            ],
            Confidence::Low,
            &[tradeoff(
                "maintainability",
                "Deep filter types can become harder to evolve for larger teams.",
            )],
        ),
        entry(
            "serde",
            "serialization",
            "Best default for broad Rust ecosystem interoperability across data formats.",
            RecommendationArchetype::BestDefault,
            &[
                "Most libraries expect or support serde.",
                "It keeps data model interop easy across application boundaries.",
            ],
            &[
                fit(
                    "boring-default",
                    GoalFitStrength::Strong,
                    "It is the default ecosystem serialization answer in most general-purpose Rust projects.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "Interoperability and derive support keep standard data handling fast to wire.",
                ),
                fit(
                    "typed-surfaces",
                    GoalFitStrength::Good,
                    "Strong typed data model integration is one of its main strengths.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "specialization",
                "Zero-copy or highly specialized binary cases may prefer narrower crates.",
            )],
        ),
        entry(
            "postcard",
            "serialization",
            "Lean option for compact binary serialization when general ecosystem interop is not the main target.",
            RecommendationArchetype::LeanOption,
            &[
                "Useful when the payload format is compact and purpose-built.",
                "Fits projects that care more about tight binary formats than broad third-party format support.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Strong,
                    "Compact binary serialization is the whole reason to choose it over serde-centric defaults.",
                ),
                fit(
                    "maximum-control",
                    GoalFitStrength::Good,
                    "It makes more sense when the format is part of a deliberate systems choice.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "interoperability",
                "It is more specialized than serde for general application interchange.",
            )],
        ),
        entry(
            "rkyv",
            "serialization",
            "Power option when archived data access and zero-copy strategies drive the design.",
            RecommendationArchetype::PowerOption,
            &[
                "Optimized for archived representations and performance-sensitive data access.",
                "Useful when serialization format decisions are part of the performance strategy itself.",
            ],
            &[
                fit(
                    "maximum-control",
                    GoalFitStrength::Strong,
                    "It is a deliberate systems choice rather than a generic application default.",
                ),
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Good,
                    "Archived representations can reduce some runtime costs when the model fits.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "interoperability",
                "It is more specialized than serde for broad ecosystem interchange.",
            )],
        ),
        entry(
            "miniserde",
            "serialization",
            "Specialist option for narrower serialization needs with minimal machinery.",
            RecommendationArchetype::Specialist,
            &[
                "Useful when reducing serialization machinery is the dominant concern.",
                "Can fit constrained cases that do not need serde's ecosystem reach.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Good,
                    "It reduces machinery when general-purpose serialization breadth is unnecessary.",
                ),
                fit(
                    "boring-default",
                    GoalFitStrength::Weak,
                    "It is not the ecosystem-default answer for shared data models.",
                ),
            ],
            Confidence::Low,
            &[tradeoff(
                "coverage",
                "It does not match serde's ecosystem reach or feature breadth.",
            )],
        ),
        entry(
            "tokio",
            "async-runtime",
            "Best default runtime for most async Rust applications.",
            RecommendationArchetype::BestDefault,
            &[
                "Broad ecosystem support makes it the least risky async default.",
                "Covers networking, timing, tasks, and many integration points in one place.",
            ],
            &[
                fit(
                    "boring-default",
                    GoalFitStrength::Strong,
                    "It is the lowest-risk async runtime default for most general application work.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Strong,
                    "It is the ecosystem center of gravity for async application crates.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "The ecosystem support reduces integration detours.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "weight",
                "It can feel heavier than smaller runtimes for narrow workloads.",
            )],
        ),
        entry(
            "smol",
            "async-runtime",
            "Lean option when a smaller async stack is the main goal.",
            RecommendationArchetype::LeanOption,
            &[
                "Useful for teams intentionally minimizing runtime surface.",
                "Can fit bespoke async stacks that value composability over all-in-one defaults.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Strong,
                    "A smaller runtime surface is the main reason to choose it over Tokio.",
                ),
                fit(
                    "maximum-control",
                    GoalFitStrength::Good,
                    "It appeals when the team wants to assemble more of the async stack deliberately.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "integration depth",
                "It usually needs more assembly than Tokio in mainstream app stacks.",
            )],
        ),
        entry(
            "async-std",
            "async-runtime",
            "Power option for teams that specifically want a std-like async API style.",
            RecommendationArchetype::PowerOption,
            &[
                "Approachable API surface for developers who like std parallels.",
                "Can still fit codebases that already standardized on it.",
            ],
            &[
                fit(
                    "maximum-control",
                    GoalFitStrength::Good,
                    "It is mostly a deliberate style choice rather than the default ecosystem path.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Good,
                    "It remains an async runtime option for teams already committed to it.",
                ),
            ],
            Confidence::Low,
            &[tradeoff(
                "ecosystem default",
                "Many async Rust libraries still target Tokio first.",
            )],
        ),
        entry(
            "embassy",
            "async-runtime",
            "Specialist option for embedded async systems rather than general server applications.",
            RecommendationArchetype::Specialist,
            &[
                "Useful when the async runtime lives inside an embedded systems context.",
                "Fits teams making a deliberate embedded async choice, not a general desktop or server default.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Good,
                    "It is relevant when constrained runtime environments matter.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Good,
                    "Async execution is still central, but in a narrower systems context.",
                ),
            ],
            Confidence::Low,
            &[tradeoff(
                "scope",
                "It is not a default answer for mainstream server-side async application stacks.",
            )],
        ),
        entry(
            "thiserror",
            "error-handling",
            "Best default for defining explicit library and application error types cleanly.",
            RecommendationArchetype::BestDefault,
            &[
                "Derive-based error types keep APIs explicit and maintainable.",
                "Strong fit when you own the public error surface and want it to stay typed.",
            ],
            &[
                fit(
                    "boring-default",
                    GoalFitStrength::Strong,
                    "It is the lowest-risk default when explicit error types are part of the design.",
                ),
                fit(
                    "typed-surfaces",
                    GoalFitStrength::Strong,
                    "Typed error surfaces are the primary value proposition.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "ergonomics",
                "Application-only contexts may prefer anyhow for faster iteration.",
            )],
        ),
        entry(
            "anyhow",
            "error-handling",
            "Lean option for application entrypoints and glue code that need ergonomic errors quickly.",
            RecommendationArchetype::LeanOption,
            &[
                "Makes application error plumbing fast and consistent.",
                "Useful when preserving typed public errors is not the main concern.",
            ],
            &[
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Strong,
                    "Application-oriented error handling moves quickly with anyhow.",
                ),
                fit(
                    "typed-surfaces",
                    GoalFitStrength::Weak,
                    "It is a weaker fit when the API contract needs explicit typed errors.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "API clarity",
                "Opaque application errors are a weaker fit for library-facing APIs.",
            )],
        ),
        entry(
            "miette",
            "error-handling",
            "Power option when rich diagnostic presentation is the main requirement.",
            RecommendationArchetype::PowerOption,
            &[
                "Useful for CLI tools and user-facing workflows that care about failure presentation.",
                "Pairs well with stacks that want richer diagnostics than bare error propagation.",
            ],
            &[
                fit(
                    "rich-diagnostics",
                    GoalFitStrength::Strong,
                    "Diagnostic presentation is the reason to start here.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "It can speed up user-facing error quality if diagnostics are already a requirement.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "surface area",
                "It adds more presentation-oriented machinery than thiserror or anyhow alone.",
            )],
        ),
        entry(
            "eyre",
            "error-handling",
            "Specialist option for report-centric application diagnostics.",
            RecommendationArchetype::Specialist,
            &[
                "Good for teams that want report-style application errors and hooks for richer handlers.",
                "Can fit CLI tools that care about polished failure output without typed public APIs.",
            ],
            &[
                fit(
                    "rich-diagnostics",
                    GoalFitStrength::Good,
                    "Report-style diagnostics are the main reason to prefer it over anyhow.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "It still keeps application error plumbing fairly direct.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "standardization",
                "anyhow tends to be the more widely expected baseline for ergonomic application errors.",
            )],
        ),
        entry(
            "rstest",
            "testing",
            "Best default when parameterized tests improve coverage without much ceremony.",
            RecommendationArchetype::BestDefault,
            &[
                "Helps scale unit tests with clearer input matrices.",
                "Easy to justify in codebases that repeat similar example-based cases.",
            ],
            &[
                fit(
                    "boring-default",
                    GoalFitStrength::Strong,
                    "It is a strong default when the problem is repeated example-based tests, not custom frameworks.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "It reduces boilerplate when many table-like test cases already exist.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "macro surface",
                "Macro-heavy tests may be harder for some teams to debug.",
            )],
        ),
        entry(
            "insta",
            "testing",
            "Lean option for snapshot-heavy tests and output contract locking.",
            RecommendationArchetype::LeanOption,
            &[
                "Useful when textual or structured output is central to correctness.",
                "Helps stabilize CLI and renderer behavior early.",
            ],
            &[
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Strong,
                    "Snapshot tests make contract coverage quick to add for output-heavy tools.",
                ),
                fit(
                    "rich-diagnostics",
                    GoalFitStrength::Good,
                    "Snapshot diffs improve failure readability for output-oriented tests.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "review hygiene",
                "Snapshots can hide behavior drift if updates are accepted casually.",
            )],
        ),
        entry(
            "proptest",
            "testing",
            "Power option when properties and randomized input spaces matter more than fixed examples.",
            RecommendationArchetype::PowerOption,
            &[
                "Excellent for invariants that are hard to enumerate manually.",
                "Useful when correctness depends on broad input coverage rather than a short example list.",
            ],
            &[
                fit(
                    "property-coverage",
                    GoalFitStrength::Strong,
                    "Property-based exploration is the reason to start here.",
                ),
                fit(
                    "maximum-control",
                    GoalFitStrength::Good,
                    "Generator control makes more sense when the team wants to model the input space deliberately.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "test complexity",
                "Shrinking and generators add setup cost over example-based tests.",
            )],
        ),
        entry(
            "quickcheck",
            "testing",
            "Specialist option for lighter property-based testing needs.",
            RecommendationArchetype::Specialist,
            &[
                "Useful when the team wants property-style tests with less machinery than proptest.",
                "Can fit smaller codebases where the heaviest generator control is unnecessary.",
            ],
            &[
                fit(
                    "property-coverage",
                    GoalFitStrength::Good,
                    "It still targets property-style coverage, just with a lighter posture than proptest.",
                ),
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Good,
                    "It is a smaller commitment than the most feature-rich property testing stacks.",
                ),
            ],
            Confidence::Low,
            &[tradeoff(
                "control depth",
                "It offers less control and shrink behavior depth than proptest.",
            )],
        ),
        entry(
            "sqlx",
            "database-access",
            "Best default for async-first SQL access with compile-time query checking.",
            RecommendationArchetype::BestDefault,
            &[
                "Strong fit when SQL remains explicit and async matters.",
                "Query checking is valuable for teams that want raw SQL with guardrails.",
            ],
            &[
                fit(
                    "boring-default",
                    GoalFitStrength::Strong,
                    "It is a strong default for async applications that still want explicit SQL.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Strong,
                    "Async database access is where it fits best.",
                ),
                fit(
                    "typed-surfaces",
                    GoalFitStrength::Good,
                    "Query checking improves confidence when explicit SQL is part of the contract.",
                ),
            ],
            Confidence::High,
            &[tradeoff(
                "compile pipeline",
                "Compile-time checking and database setup can complicate builds.",
            )],
        ),
        entry(
            "tokio-postgres",
            "database-access",
            "Lean option when you want direct async PostgreSQL access with less ORM machinery.",
            RecommendationArchetype::LeanOption,
            &[
                "Good when the team wants a direct driver instead of a broader abstraction layer.",
                "Useful for services that keep SQL and connection handling explicit.",
            ],
            &[
                fit(
                    "minimal-footprint",
                    GoalFitStrength::Good,
                    "It stays closer to a driver than the broader query and ORM stacks.",
                ),
                fit(
                    "maximum-control",
                    GoalFitStrength::Good,
                    "It keeps database behavior explicit instead of hiding it behind a larger abstraction.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Good,
                    "It remains a fit for async-native service stacks.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "portability",
                "It is narrower in scope than cross-database abstractions or ORMs.",
            )],
        ),
        entry(
            "diesel",
            "database-access",
            "Power option when strong schema typing and an ORM-style workflow are part of the design.",
            RecommendationArchetype::PowerOption,
            &[
                "Schema-driven modeling can be valuable in traditional server applications.",
                "Useful when the team already likes Diesel's query builder and migration workflow.",
            ],
            &[
                fit(
                    "typed-surfaces",
                    GoalFitStrength::Strong,
                    "Schema typing is the main argument for choosing it over lighter async SQL layers.",
                ),
                fit(
                    "maximum-control",
                    GoalFitStrength::Good,
                    "It is a deliberate database modeling choice rather than the lightest path.",
                ),
                fit(
                    "async",
                    GoalFitStrength::Weak,
                    "Async-first application stacks often find other choices easier to adopt.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "async ergonomics",
                "Async-first applications often find sqlx or a direct driver easier to adopt.",
            )],
        ),
        entry(
            "sea-orm",
            "database-access",
            "Specialist option for async teams that want higher-level entity modeling.",
            RecommendationArchetype::Specialist,
            &[
                "Useful when an async ORM is preferable to handwritten SQL.",
                "Can speed up CRUD-heavy service development when the team wants more abstraction.",
            ],
            &[
                fit(
                    "async",
                    GoalFitStrength::Good,
                    "It still targets async application stacks rather than blocking-only workloads.",
                ),
                fit(
                    "fastest-to-ship",
                    GoalFitStrength::Good,
                    "The higher-level model can speed up CRUD-heavy application work.",
                ),
            ],
            Confidence::Medium,
            &[tradeoff(
                "abstraction cost",
                "Higher-level modeling can obscure SQL behavior and tuning tradeoffs.",
            )],
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn entry(
    crate_name: &str,
    intent: &str,
    summary: &str,
    archetype: RecommendationArchetype,
    rationale: &[&str],
    goal_fits: &[GoalFit],
    confidence: Confidence,
    tradeoffs: &[Tradeoff],
) -> CatalogEntry {
    CatalogEntry {
        crate_name: crate_name.to_string(),
        intent: intent.to_string(),
        summary: summary.to_string(),
        rationale: rationale.iter().map(|line| (*line).to_string()).collect(),
        goal_fits: goal_fits.to_vec(),
        tradeoffs: tradeoffs.to_vec(),
        trust_notes: vec![TrustNote {
            label: "curated catalog".to_string(),
            detail:
                "This recommendation comes from a checked-in phase-2 catalog, not live ecosystem telemetry."
                    .to_string(),
        }],
        confidence,
        archetype,
    }
}

fn fit(goal: &str, strength: GoalFitStrength, detail: &str) -> GoalFit {
    GoalFit {
        goal: goal.to_string(),
        strength,
        detail: detail.to_string(),
    }
}

fn tradeoff(area: &str, detail: &str) -> Tradeoff {
    Tradeoff {
        area: area.to_string(),
        detail: detail.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::seed_catalog;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn required_intents_are_seeded() {
        let intents: BTreeSet<String> = seed_catalog()
            .into_iter()
            .map(|entry| entry.intent)
            .collect();
        let required = [
            "cli-parsing",
            "config",
            "logging-tracing",
            "http-client",
            "http-server",
            "serialization",
            "async-runtime",
            "error-handling",
            "testing",
            "database-access",
        ];

        for intent in required {
            assert!(intents.contains(intent), "missing intent: {intent}");
        }
    }

    #[test]
    fn every_intent_has_at_least_four_curated_candidates() {
        let mut counts = BTreeMap::new();
        for entry in seed_catalog() {
            *counts.entry(entry.intent).or_insert(0usize) += 1;
            assert!(
                !entry.goal_fits.is_empty(),
                "catalog entry {} should carry goal fits",
                entry.crate_name
            );
        }

        for (intent, count) in counts {
            assert!(count >= 4, "{intent} only has {count} entries");
        }
    }
}
