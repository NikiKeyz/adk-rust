//! Provider configuration for the composable scaffolding engine.
//!
//! Each LLM provider has a configuration that determines the feature flag,
//! environment variable, model initialization code, and default model.

/// Provider-specific configuration for code generation.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Provider name (e.g., "gemini", "openai").
    pub name: &'static str,
    /// Cargo feature flag to enable this provider.
    pub feature_flag: &'static str,
    /// Environment variable for the API key or endpoint.
    pub env_var: &'static str,
    /// Code snippet for model initialization in `main.rs`.
    pub model_init_code: &'static str,
    /// Default model identifier.
    pub default_model: &'static str,
    /// Whether this provider requires an API key.
    pub requires_api_key: bool,
}

/// All supported provider configurations.
static PROVIDERS: &[ProviderConfig] = &[
    ProviderConfig {
        name: "gemini",
        feature_flag: "gemini",
        env_var: "GOOGLE_API_KEY",
        model_init_code: "adk_rust::model::GeminiModel::new(&api_key, \"gemini-3.5-flash\")?",
        default_model: "gemini-3.5-flash",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "openai",
        feature_flag: "openai",
        env_var: "OPENAI_API_KEY",
        model_init_code: "adk_rust::model::openai::OpenAIClient::new(\n        adk_rust::model::openai::OpenAIConfig::new(&api_key, \"gpt-5.5\"),\n    )?",
        default_model: "gpt-5.5",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "anthropic",
        feature_flag: "anthropic",
        env_var: "ANTHROPIC_API_KEY",
        model_init_code: "adk_rust::model::anthropic::AnthropicClient::new(\n        adk_rust::model::anthropic::AnthropicConfig::new(&api_key, \"claude-sonnet-4-6\"),\n    )?",
        default_model: "claude-sonnet-4-6",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "deepseek",
        feature_flag: "deepseek",
        env_var: "DEEPSEEK_API_KEY",
        model_init_code: "adk_rust::model::deepseek::DeepSeekClient::new(\n        adk_rust::model::deepseek::DeepSeekConfig::new(&api_key, \"deepseek-v4-flash\"),\n    )?",
        default_model: "deepseek-v4-flash",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "ollama",
        feature_flag: "ollama",
        env_var: "",
        model_init_code: "adk_rust::model::ollama::OllamaModel::new(\n        adk_rust::model::ollama::OllamaConfig::new(\"gemma4\"),\n    )?",
        default_model: "gemma4",
        requires_api_key: false,
    },
    ProviderConfig {
        name: "groq",
        feature_flag: "groq",
        env_var: "GROQ_API_KEY",
        model_init_code: "adk_rust::model::groq::GroqClient::new(\n        adk_rust::model::groq::GroqConfig::new(&api_key, \"llama-3.3-70b-versatile\"),\n    )?",
        default_model: "llama-3.3-70b-versatile",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "openrouter",
        feature_flag: "openrouter",
        env_var: "OPENROUTER_API_KEY",
        model_init_code: "adk_rust::model::openrouter::OpenRouterClient::new(\n        adk_rust::model::openrouter::OpenRouterConfig::new(&api_key, \"qwen/qwen3.7-max\"),\n    )?",
        default_model: "qwen/qwen3.7-max",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "bedrock",
        feature_flag: "bedrock",
        env_var: "AWS_REGION",
        model_init_code: "adk_rust::model::bedrock::BedrockClient::new(\n        adk_rust::model::bedrock::BedrockConfig::new(\n            std::env::var(\"AWS_REGION\").unwrap_or_else(|_| \"us-east-1\".to_string()),\n            \"anthropic.claude-opus-4-6-v1\",\n        ),\n    ).await?",
        default_model: "anthropic.claude-opus-4-6-v1",
        requires_api_key: false,
    },
    ProviderConfig {
        name: "azure-ai",
        feature_flag: "azure-ai",
        env_var: "AZURE_AI_KEY",
        model_init_code: "adk_rust::model::azure_ai::AzureAIClient::new(\n        adk_rust::model::azure_ai::AzureAIConfig::new(\n            std::env::var(\"AZURE_AI_ENDPOINT\").expect(\"AZURE_AI_ENDPOINT must be set\"),\n            &api_key,\n            \"gpt-5.5\",\n        ),\n    )?",
        default_model: "gpt-5.5",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "xai",
        feature_flag: "openai",
        env_var: "XAI_API_KEY",
        model_init_code: "adk_rust::model::OpenAICompatible::new(\n        adk_rust::model::OpenAICompatibleConfig::xai(&api_key, \"grok-4.3\"),\n    )?",
        default_model: "grok-4.3",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "mistral",
        feature_flag: "openai",
        env_var: "MISTRAL_API_KEY",
        model_init_code: "adk_rust::model::OpenAICompatible::new(\n        adk_rust::model::OpenAICompatibleConfig::mistral(&api_key, \"mistral-large-latest\"),\n    )?",
        default_model: "mistral-large-latest",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "perplexity",
        feature_flag: "openai",
        env_var: "PERPLEXITY_API_KEY",
        model_init_code: "adk_rust::model::OpenAICompatible::new(\n        adk_rust::model::OpenAICompatibleConfig::perplexity(&api_key, \"sonar-pro\"),\n    )?",
        default_model: "sonar-pro",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "minimax",
        feature_flag: "openai",
        env_var: "MINIMAX_API_KEY",
        model_init_code: "adk_rust::model::OpenAICompatible::new(\n        adk_rust::model::OpenAICompatibleConfig::minimax(&api_key, \"minimax-m2.7\"),\n    )?",
        default_model: "minimax-m2.7",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "bytedance",
        feature_flag: "openai",
        env_var: "ARK_API_KEY",
        model_init_code: "adk_rust::model::OpenAICompatible::new(\n        adk_rust::model::OpenAICompatibleConfig::bytedance(&api_key, \"doubao-1-5-pro-256k\"),\n    )?",
        default_model: "doubao-1-5-pro-256k",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "zhipu",
        feature_flag: "openai",
        env_var: "ZHIPU_API_KEY",
        model_init_code: "adk_rust::model::OpenAICompatible::new(\n        adk_rust::model::OpenAICompatibleConfig::zhipu(&api_key, \"glm-5.1\"),\n    )?",
        default_model: "glm-5.1",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "baidu",
        feature_flag: "openai",
        env_var: "QIANFAN_API_KEY",
        model_init_code: "adk_rust::model::OpenAICompatible::new(\n        adk_rust::model::OpenAICompatibleConfig::baidu(&api_key, \"ernie-5\"),\n    )?",
        default_model: "ernie-5",
        requires_api_key: true,
    },
    ProviderConfig {
        name: "cohere",
        feature_flag: "openai",
        env_var: "COHERE_API_KEY",
        model_init_code: "adk_rust::model::OpenAICompatible::new(\n        adk_rust::model::OpenAICompatibleConfig::cohere(&api_key, \"command-a-plus-05-2026\"),\n    )?",
        default_model: "command-a-plus-05-2026",
        requires_api_key: true,
    },
];

/// Look up a provider configuration by name.
///
/// # Errors
///
/// Returns an error string if the provider name is not recognized.
pub fn get_provider_config(provider: &str) -> Result<&'static ProviderConfig, String> {
    PROVIDERS.iter().find(|p| p.name == provider).ok_or_else(|| {
        let supported: Vec<&str> = PROVIDERS.iter().map(|p| p.name).collect();
        format!("unknown provider '{provider}'. Supported: {}", supported.join(", "))
    })
}

/// Returns all registered provider configurations.
pub fn all_providers() -> &'static [ProviderConfig] {
    PROVIDERS
}
