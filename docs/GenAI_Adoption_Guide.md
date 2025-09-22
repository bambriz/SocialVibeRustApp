# Adopting Generative AI for Emotion and Content Moderation

## Table of Contents
1. [Problem Framing](#problem-framing)
2. [Current Architecture Analysis](#current-architecture-analysis)
3. [Gen AI Options Matrix](#gen-ai-options-matrix)
4. [Implementation Strategies](#implementation-strategies)
5. [Model Selection & Evaluation](#model-selection--evaluation)
6. [Safety & Policy Framework](#safety--policy-framework)
7. [Reliability & Performance](#reliability--performance)
8. [Security & Privacy](#security--privacy)
9. [Observability & Monitoring](#observability--monitoring)
10. [Migration & Rollout Plan](#migration--rollout-plan)
11. [Appendix: Code Examples](#appendix-code-examples)

## Problem Framing

### Current Capabilities
Social Pulse currently uses traditional ML models for emotion detection and content moderation:
- **Sentiment Analysis**: HuggingFace EmotionClassifier with custom pattern detection
- **Content Moderation**: HateSonar + TextBlob for toxicity detection
- **Deployment**: Python subprocess with HTTP API

### Generative AI Opportunities

#### Enhanced Emotion Detection
- **Nuanced Emotion Understanding**: LLMs can detect subtle emotional contexts, sarcasm, cultural nuances
- **Multi-dimensional Analysis**: Simultaneous detection of multiple emotions with confidence scores
- **Context Awareness**: Understanding emotions in conversation threads and reply contexts
- **Cultural Sensitivity**: Better handling of cultural and linguistic variations in emotional expression

#### Advanced Content Moderation
- **Contextual Understanding**: Detect harmful content that traditional keyword-based systems miss
- **Intent Recognition**: Distinguish between harmful intent and legitimate discussion
- **Explanation Generation**: Provide clear, human-readable explanations for moderation decisions
- **Policy Adaptation**: Easily update moderation policies without retraining models
- **False Positive Reduction**: Better accuracy in distinguishing edge cases

### Business Benefits
- **Improved User Experience**: More accurate emotion detection leads to better content recommendations
- **Reduced Moderation Overhead**: Fewer false positives mean less manual review required
- **Enhanced Safety**: Better detection of subtle forms of harassment and harmful content
- **Transparency**: AI explanations help users understand moderation decisions
- **Scalability**: Cloud-based AI services can handle traffic spikes without infrastructure scaling

## Current Architecture Analysis

### Existing Python Service Architecture

```python
# Current Sentiment Analysis Service
class SentimentAnalyzer:
    def analyze(self, text: str) -> dict:
        # HuggingFace EmotionClassifier + pattern detection
        emotions = self.emotion_classifier(text)
        patterns = self.detect_patterns(text)
        return self.combine_results(emotions, patterns)

# Current Moderation Service  
class ContentModerator:
    def moderate(self, text: str) -> dict:
        # HateSonar + TextBlob analysis
        toxicity = self.hate_classifier(text)
        sentiment = self.sentiment_analyzer(text)
        return self.make_decision(toxicity, sentiment)
```

### Current Service Interface

```rust
// Rust service interface
pub struct SentimentService {
    // Calls Python subprocess via HTTP
}

impl SentimentService {
    pub async fn analyze_sentiment(&self, text: &str) -> Result<Vec<Sentiment>, Error> {
        // HTTP call to Python service
        // Parse response into Sentiment struct
    }
}

pub struct ModerationService {
    // Calls Python subprocess via HTTP
}

impl ModerationService {
    pub async fn check_content(&self, text: &str) -> Result<ModerationResult, Error> {
        // HTTP call to Python service
        // Parse response into ModerationResult struct
    }
}
```

### Integration Points
- **Real-time Sentiment Preview**: Frontend → Rust API → Python Service
- **Post Creation**: Content → Moderation Check → Sentiment Analysis → Database Storage
- **Comment Processing**: Similar pipeline for comment creation
- **Bulk Analysis**: Background processing for existing content

## Gen AI Options Matrix

### Option 1: Cloud-Based LLM APIs

#### Azure OpenAI Service
**Pros:**
- Enterprise-grade security and compliance
- Microsoft's responsible AI framework
- Integrated with Azure ecosystem
- Content filtering built-in
- Regional deployment options

**Cons:**
- Higher cost per request
- Requires Azure subscription
- Potential vendor lock-in
- Rate limiting considerations

**Use Case Fit:** Best for enterprise deployments requiring compliance and security

#### OpenAI API (GPT-4/GPT-3.5)
**Pros:**
- State-of-the-art model performance
- Extensive documentation and community
- Function calling capabilities
- Fast inference times

**Cons:**
- Data privacy concerns
- Usage policies may restrict content types
- Cost scaling with volume
- No guarantee of data residency

**Use Case Fit:** Ideal for rapid prototyping and high-quality results

#### Google Vertex AI
**Pros:**
- Comprehensive ML platform
- Custom model training options
- Strong safety features
- Competitive pricing

**Cons:**
- Steeper learning curve
- Less mature than OpenAI for text generation
- Google Cloud dependency

**Use Case Fit:** Good for organizations already using Google Cloud

### Option 2: Self-Hosted Open Source Models

#### Hugging Face Transformers
**Models:** Meta Llama 2/3, Mistral, Claude-style models

**Pros:**
- Complete data control
- No per-request costs after setup
- Customizable for specific use cases
- No external dependencies

**Cons:**
- Significant infrastructure requirements
- Model management complexity
- Need for ML expertise
- Inference optimization challenges

**Use Case Fit:** Organizations with ML expertise and infrastructure

#### Local GPU Deployment
**Pros:**
- Maximum privacy and control
- Predictable costs
- No network latency

**Cons:**
- High upfront hardware costs
- Limited scaling capabilities
- Maintenance overhead

**Use Case Fit:** Smaller deployments with strict privacy requirements

### Option 3: Hybrid Approach

#### Primary + Fallback Strategy
- **Primary**: Cloud API for production traffic
- **Fallback**: Local model for API failures
- **Benefits**: Reliability with cost optimization

#### Workload Distribution
- **Real-time**: Fast cloud APIs for user-facing features
- **Batch**: Self-hosted models for background processing
- **Benefits**: Optimize cost and performance by use case

### Recommendation Matrix

| Criteria | Azure OpenAI | OpenAI API | Vertex AI | Self-Hosted | Hybrid |
|----------|--------------|------------|-----------|-------------|--------|
| **Security** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Performance** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Cost** | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Ease of Use** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| **Privacy** | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |

## Implementation Strategies

### Strategy 1: Service Interface Abstraction

#### Create AI Service Trait
```rust
#[async_trait]
pub trait AIService: Send + Sync {
    async fn analyze_sentiment(&self, text: &str, context: Option<&str>) -> Result<SentimentAnalysis, AIError>;
    async fn moderate_content(&self, text: &str, context: Option<&str>) -> Result<ModerationResult, AIError>;
    async fn explain_decision(&self, text: &str, decision: &str) -> Result<String, AIError>;
}

// Current implementation
pub struct PythonAIService {
    client: HttpClient,
    base_url: String,
}

// New Gen AI implementation
pub struct OpenAIService {
    client: openai::Client,
    model: String,
}

// Implementation factory
pub fn create_ai_service(config: &AIConfig) -> Box<dyn AIService> {
    match config.provider {
        AIProvider::Python => Box::new(PythonAIService::new(config)),
        AIProvider::OpenAI => Box::new(OpenAIService::new(config)),
        AIProvider::Azure => Box::new(AzureOpenAIService::new(config)),
    }
}
```

#### Enhanced Data Models
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SentimentAnalysis {
    pub primary_emotion: Emotion,
    pub emotion_scores: HashMap<Emotion, f64>,
    pub confidence: f64,
    pub explanation: Option<String>,
    pub context_factors: Vec<String>,
    pub cultural_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModerationResult {
    pub is_blocked: bool,
    pub risk_level: RiskLevel, // Low, Medium, High, Critical
    pub violation_categories: Vec<ViolationCategory>,
    pub confidence: f64,
    pub explanation: String,
    pub suggested_action: ModerationAction,
    pub appeal_guidance: Option<String>,
}
```

### Strategy 2: Prompt Engineering Framework

#### Sentiment Analysis Prompts
```python
SENTIMENT_ANALYSIS_PROMPT = """
Analyze the emotional content of the following text. Consider:
- Primary and secondary emotions
- Intensity levels
- Cultural context and nuances
- Sarcasm or irony
- Implicit emotions

Text: "{text}"
Context: {context}

Provide analysis in this JSON format:
{{
    "primary_emotion": "emotion_name",
    "emotion_scores": {{
        "joy": 0.0-1.0,
        "anger": 0.0-1.0,
        "sadness": 0.0-1.0,
        "fear": 0.0-1.0,
        "disgust": 0.0-1.0,
        "surprise": 0.0-1.0
    }},
    "confidence": 0.0-1.0,
    "explanation": "Brief explanation of the emotional analysis",
    "context_factors": ["factor1", "factor2"],
    "cultural_notes": "Any cultural considerations"
}}
"""
```

#### Content Moderation Prompts
```python
MODERATION_PROMPT = """
Evaluate the following content for potential policy violations. Consider:
- Harmful intent vs. legitimate discussion
- Context and cultural factors
- Severity of potential harm
- Age-appropriateness

Content: "{text}"
Context: {context}
Platform Policy: {policy_summary}

Provide analysis in this JSON format:
{{
    "is_blocked": boolean,
    "risk_level": "low|medium|high|critical",
    "violation_categories": ["harassment", "hate_speech", "violence", "adult_content"],
    "confidence": 0.0-1.0,
    "explanation": "Clear explanation of the decision",
    "suggested_action": "allow|warn|moderate|block",
    "appeal_guidance": "How users can appeal if they disagree"
}}
"""
```

### Strategy 3: Gradual Migration Plan

#### Phase 1: Parallel Processing (Week 1-2)
- Run both traditional ML and Gen AI in parallel
- Log results for comparison
- No user-facing changes
- Gather performance metrics

#### Phase 2: Shadow Mode (Week 3-4)
- Gen AI processes all content
- Traditional ML serves user-facing results
- Build confidence in Gen AI accuracy
- Tune prompts based on results

#### Phase 3: Gradual Rollout (Week 5-8)
- Start with 5% of traffic to Gen AI
- Gradually increase based on success metrics
- Monitor error rates and user feedback
- Maintain rollback capability

#### Phase 4: Full Migration (Week 9-10)
- Switch primary traffic to Gen AI
- Keep traditional ML as fallback
- Monitor performance closely
- Optimize based on production usage

## Model Selection & Evaluation

### Evaluation Framework

#### Sentiment Analysis Metrics
```python
class SentimentEvaluator:
    def __init__(self):
        self.metrics = {
            'accuracy': [],
            'precision': [],
            'recall': [],
            'f1_score': [],
            'emotion_correlation': [],
            'cultural_sensitivity': [],
            'response_time': []
        }
    
    def evaluate_model(self, model, test_dataset):
        results = []
        for sample in test_dataset:
            prediction = model.predict(sample.text)
            ground_truth = sample.emotions
            
            # Calculate metrics
            accuracy = self.calculate_accuracy(prediction, ground_truth)
            cultural_score = self.assess_cultural_sensitivity(sample, prediction)
            
            results.append({
                'accuracy': accuracy,
                'cultural_sensitivity': cultural_score,
                'response_time': prediction.response_time
            })
        
        return self.aggregate_results(results)
```

#### Content Moderation Metrics
```python
class ModerationEvaluator:
    def __init__(self):
        self.metrics = {
            'precision': [],  # True positives / (True positives + False positives)
            'recall': [],     # True positives / (True positives + False negatives)
            'specificity': [], # True negatives / (True negatives + False positives)
            'false_positive_rate': [],
            'explanation_quality': [],
            'consistency': []
        }
    
    def evaluate_explanations(self, decisions):
        """Evaluate quality of AI explanations"""
        scores = []
        for decision in decisions:
            explanation = decision.explanation
            # Check for clarity, relevance, and helpfulness
            clarity_score = self.assess_clarity(explanation)
            relevance_score = self.assess_relevance(explanation, decision.content)
            helpfulness_score = self.assess_helpfulness(explanation)
            
            scores.append((clarity_score + relevance_score + helpfulness_score) / 3)
        
        return np.mean(scores)
```

### Test Dataset Curation

#### Emotion Detection Test Cases
- **Basic Emotions**: Clear examples of joy, sadness, anger, fear, disgust, surprise
- **Complex Emotions**: Mixed emotions, sarcasm, irony, cultural expressions
- **Edge Cases**: Ambiguous content, context-dependent emotions
- **Multilingual**: Content in different languages and cultural contexts
- **Conversation Context**: Replies and threaded conversations

#### Content Moderation Test Cases
- **Clear Violations**: Obvious harassment, hate speech, threats
- **Edge Cases**: Borderline content, context-dependent violations
- **False Positives**: Legitimate content that might be misclassified
- **Cultural Sensitivity**: Content that varies by cultural context
- **Discussion Topics**: Academic discussions of sensitive topics

### Model Comparison Framework

```python
class ModelComparison:
    def __init__(self):
        self.models = {}
        self.test_datasets = {}
    
    def add_model(self, name: str, model_instance):
        self.models[name] = model_instance
    
    def run_comparison(self, task_type: str):
        results = {}
        dataset = self.test_datasets[task_type]
        
        for model_name, model in self.models.items():
            print(f"Evaluating {model_name}...")
            
            start_time = time.time()
            predictions = []
            
            for sample in dataset:
                if task_type == 'sentiment':
                    prediction = model.analyze_sentiment(sample.text)
                elif task_type == 'moderation':
                    prediction = model.moderate_content(sample.text)
                
                predictions.append(prediction)
            
            evaluation_time = time.time() - start_time
            
            # Calculate metrics
            accuracy = self.calculate_accuracy(predictions, dataset)
            precision = self.calculate_precision(predictions, dataset)
            recall = self.calculate_recall(predictions, dataset)
            
            results[model_name] = {
                'accuracy': accuracy,
                'precision': precision,
                'recall': recall,
                'avg_response_time': evaluation_time / len(dataset),
                'total_evaluation_time': evaluation_time
            }
        
        return results
```

## Safety & Policy Framework

### Content Policy Mapping

#### Policy Categories
```yaml
moderation_policies:
  harassment:
    description: "Targeted harassment or bullying"
    severity_levels:
      low: "Mild teasing or criticism"
      medium: "Persistent negative targeting"
      high: "Severe personal attacks"
      critical: "Threats of violence or self-harm"
    actions:
      low: "warning"
      medium: "content_removal"
      high: "temporary_suspension"
      critical: "permanent_ban"
  
  hate_speech:
    description: "Content attacking individuals or groups"
    protected_characteristics:
      - race
      - religion
      - gender
      - sexual_orientation
      - disability
    context_considerations:
      - academic_discussion
      - news_reporting
      - historical_context
  
  misinformation:
    description: "False or misleading information"
    categories:
      - health_misinformation
      - election_misinformation
      - climate_denial
    verification_sources:
      - fact_checking_organizations
      - authoritative_institutions
```

#### Policy Implementation
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub category: ViolationCategory,
    pub severity: SeverityLevel,
    pub confidence: f64,
    pub context_factors: Vec<String>,
    pub recommended_action: ModerationAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ModerationAction {
    Allow,
    Warning { message: String },
    ContentRemoval { reason: String },
    TemporarySuspension { duration_hours: u32, reason: String },
    PermanentBan { reason: String },
    HumanReview { priority: Priority },
}

pub struct PolicyEngine {
    policies: HashMap<ViolationCategory, Policy>,
    escalation_rules: Vec<EscalationRule>,
}

impl PolicyEngine {
    pub fn evaluate_violation(&self, violation: &PolicyViolation, user_history: &UserHistory) -> ModerationAction {
        let base_action = self.get_base_action(violation);
        let escalated_action = self.apply_escalation_rules(base_action, user_history);
        
        // Consider appeals and context
        self.apply_context_adjustments(escalated_action, violation)
    }
}
```

### Bias Mitigation Strategies

#### Bias Detection Framework
```python
class BiasDetector:
    def __init__(self):
        self.protected_groups = [
            'race', 'gender', 'religion', 'age', 'disability',
            'sexual_orientation', 'political_affiliation'
        ]
    
    def detect_sentiment_bias(self, model, test_cases):
        """Detect bias in sentiment analysis across different groups"""
        bias_results = {}
        
        for group in self.protected_groups:
            group_cases = [case for case in test_cases if case.group == group]
            control_cases = [case for case in test_cases if case.group == 'control']
            
            group_scores = [model.analyze_sentiment(case.text).primary_score 
                          for case in group_cases]
            control_scores = [model.analyze_sentiment(case.text).primary_score 
                            for case in control_cases]
            
            # Statistical test for bias
            bias_score = self.calculate_bias_score(group_scores, control_scores)
            bias_results[group] = bias_score
        
        return bias_results
    
    def detect_moderation_bias(self, model, test_cases):
        """Detect bias in content moderation across different groups"""
        bias_results = {}
        
        for group in self.protected_groups:
            group_cases = [case for case in test_cases if case.group == group]
            control_cases = [case for case in test_cases if case.group == 'control']
            
            group_block_rate = len([case for case in group_cases 
                                  if model.moderate_content(case.text).is_blocked]) / len(group_cases)
            control_block_rate = len([case for case in control_cases 
                                    if model.moderate_content(case.text).is_blocked]) / len(control_cases)
            
            bias_ratio = group_block_rate / control_block_rate if control_block_rate > 0 else float('inf')
            bias_results[group] = bias_ratio
        
        return bias_results
```

#### Bias Mitigation Techniques
1. **Prompt Engineering**: Include bias-awareness instructions in prompts
2. **Representative Training Data**: Ensure diverse training examples
3. **Regular Bias Audits**: Automated testing for bias in production
4. **Human Review**: Escalate edge cases to human moderators
5. **Community Feedback**: Allow users to report biased decisions

### Fail-Safe Mechanisms

#### Fail-Open vs Fail-Closed Strategies
```rust
pub struct FailSafeConfig {
    pub sentiment_analysis_fallback: FailureMode,
    pub content_moderation_fallback: FailureMode,
    pub explanation_generation_fallback: FailureMode,
}

#[derive(Debug, Clone)]
pub enum FailureMode {
    FailOpen,   // Allow content when AI fails
    FailClosed, // Block content when AI fails
    UseFallback(Box<dyn AIService>), // Use backup service
    RequireHumanReview, // Escalate to human moderators
}

pub struct ResilientAIService {
    primary_service: Box<dyn AIService>,
    fallback_service: Option<Box<dyn AIService>>,
    config: FailSafeConfig,
}

impl ResilientAIService {
    pub async fn moderate_content(&self, text: &str) -> Result<ModerationResult, AIError> {
        match self.primary_service.moderate_content(text).await {
            Ok(result) => Ok(result),
            Err(error) => {
                warn!("Primary AI service failed: {}", error);
                
                match self.config.content_moderation_fallback {
                    FailureMode::FailOpen => Ok(ModerationResult::allow_with_warning()),
                    FailureMode::FailClosed => Ok(ModerationResult::block_with_reason("AI service unavailable")),
                    FailureMode::UseFallback(ref fallback) => {
                        fallback.moderate_content(text).await
                            .unwrap_or_else(|_| ModerationResult::require_human_review())
                    },
                    FailureMode::RequireHumanReview => Ok(ModerationResult::require_human_review()),
                }
            }
        }
    }
}
```

## Reliability & Performance

### Timeout and Retry Strategies

#### Adaptive Timeout Configuration
```rust
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    pub sentiment_analysis: Duration,
    pub content_moderation: Duration,
    pub explanation_generation: Duration,
    pub max_retries: u32,
    pub retry_backoff: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            sentiment_analysis: Duration::from_secs(2),
            content_moderation: Duration::from_secs(3),
            explanation_generation: Duration::from_secs(5),
            max_retries: 3,
            retry_backoff: Duration::from_millis(500),
        }
    }
}

pub struct ReliableAIClient {
    client: reqwest::Client,
    config: TimeoutConfig,
    metrics: Arc<Mutex<ClientMetrics>>,
}

impl ReliableAIClient {
    pub async fn call_with_retry<T>(
        &self,
        request: impl Fn() -> BoxFuture<'_, Result<T, AIError>>,
        timeout: Duration,
    ) -> Result<T, AIError> {
        let mut last_error = None;
        
        for attempt in 1..=self.config.max_retries {
            let result = timeout(timeout, request()).await;
            
            match result {
                Ok(Ok(response)) => {
                    self.record_success(attempt).await;
                    return Ok(response);
                },
                Ok(Err(error)) => {
                    last_error = Some(error);
                    if !error.is_retryable() {
                        break;
                    }
                },
                Err(_) => {
                    last_error = Some(AIError::Timeout);
                }
            }
            
            if attempt < self.config.max_retries {
                let backoff = self.config.retry_backoff * attempt;
                tokio::time::sleep(backoff).await;
            }
        }
        
        self.record_failure(last_error.as_ref()).await;
        Err(last_error.unwrap_or(AIError::MaxRetriesExceeded))
    }
}
```

### Caching Strategies

#### Response Caching
```rust
use moka::future::Cache;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct CacheKey {
    pub content_hash: u64,
    pub model_version: String,
    pub analysis_type: AnalysisType,
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content_hash.hash(state);
        self.model_version.hash(state);
        self.analysis_type.hash(state);
    }
}

pub struct CachedAIService {
    inner: Box<dyn AIService>,
    sentiment_cache: Cache<CacheKey, SentimentAnalysis>,
    moderation_cache: Cache<CacheKey, ModerationResult>,
}

impl CachedAIService {
    pub fn new(inner: Box<dyn AIService>) -> Self {
        Self {
            inner,
            sentiment_cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_hours(24))
                .build(),
            moderation_cache: Cache::builder()
                .max_capacity(10_000)
                .time_to_live(Duration::from_hours(6))
                .build(),
        }
    }
    
    fn content_hash(text: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }
}

#[async_trait]
impl AIService for CachedAIService {
    async fn analyze_sentiment(&self, text: &str, context: Option<&str>) -> Result<SentimentAnalysis, AIError> {
        let cache_key = CacheKey {
            content_hash: Self::content_hash(text),
            model_version: "v1.0".to_string(),
            analysis_type: AnalysisType::Sentiment,
        };
        
        if let Some(cached_result) = self.sentiment_cache.get(&cache_key).await {
            return Ok(cached_result);
        }
        
        let result = self.inner.analyze_sentiment(text, context).await?;
        self.sentiment_cache.insert(cache_key, result.clone()).await;
        
        Ok(result)
    }
}
```

### Cold Start Mitigation

#### Predictive Preloading
```rust
pub struct PredictivePreloader {
    ai_service: Box<dyn AIService>,
    warmup_content: Vec<String>,
    preload_scheduler: tokio::task::JoinHandle<()>,
}

impl PredictivePreloader {
    pub fn new(ai_service: Box<dyn AIService>) -> Self {
        let warmup_content = vec![
            "This is a test message for warming up the AI service.".to_string(),
            "Another example to ensure the service is responsive.".to_string(),
        ];
        
        let service_clone = ai_service.clone();
        let content_clone = warmup_content.clone();
        
        let preload_scheduler = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_minutes(30));
            
            loop {
                interval.tick().await;
                
                for content in &content_clone {
                    let _ = service_clone.analyze_sentiment(content, None).await;
                    let _ = service_clone.moderate_content(content, None).await;
                }
            }
        });
        
        Self {
            ai_service,
            warmup_content,
            preload_scheduler,
        }
    }
    
    pub async fn warmup(&self) -> Result<(), AIError> {
        info!("Warming up AI service...");
        
        for content in &self.warmup_content {
            self.ai_service.analyze_sentiment(content, None).await?;
            self.ai_service.moderate_content(content, None).await?;
        }
        
        info!("AI service warmup completed");
        Ok(())
    }
}
```

## Security & Privacy

### API Key Management

#### Secure Configuration
```rust
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct AIProviderConfig {
    pub provider: AIProvider,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub model: String,
    pub region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AIProvider {
    OpenAI,
    Azure,
    Anthropic,
    HuggingFace,
    Local,
}

impl AIProviderConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let provider = env::var("AI_PROVIDER")
            .map_err(|_| ConfigError::MissingEnvironmentVariable("AI_PROVIDER".to_string()))?;
        
        match provider.as_str() {
            "openai" => Ok(Self {
                provider: AIProvider::OpenAI,
                api_key: env::var("OPENAI_API_KEY").ok(),
                endpoint: env::var("OPENAI_ENDPOINT").ok(),
                model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
                region: None,
            }),
            "azure" => Ok(Self {
                provider: AIProvider::Azure,
                api_key: env::var("AZURE_OPENAI_API_KEY").ok(),
                endpoint: env::var("AZURE_OPENAI_ENDPOINT").ok(),
                model: env::var("AZURE_OPENAI_MODEL").unwrap_or_else(|_| "gpt-35-turbo".to_string()),
                region: env::var("AZURE_REGION").ok(),
            }),
            _ => Err(ConfigError::InvalidProvider(provider)),
        }
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        match self.provider {
            AIProvider::OpenAI | AIProvider::Azure => {
                if self.api_key.is_none() {
                    return Err(ConfigError::MissingApiKey);
                }
            },
            AIProvider::Local => {
                // No API key required for local models
            },
            _ => {}
        }
        
        Ok(())
    }
}
```

#### Secret Rotation
```rust
pub struct RotatingKeyManager {
    current_key: Arc<Mutex<String>>,
    backup_key: Arc<Mutex<Option<String>>>,
    rotation_interval: Duration,
    key_provider: Box<dyn KeyProvider>,
}

#[async_trait]
pub trait KeyProvider: Send + Sync {
    async fn get_primary_key(&self) -> Result<String, KeyError>;
    async fn get_backup_key(&self) -> Result<Option<String>, KeyError>;
    async fn rotate_keys(&self) -> Result<(), KeyError>;
}

impl RotatingKeyManager {
    pub async fn start_rotation(&self) {
        let mut interval = tokio::time::interval(self.rotation_interval);
        
        loop {
            interval.tick().await;
            
            match self.key_provider.rotate_keys().await {
                Ok(()) => {
                    info!("API keys rotated successfully");
                    
                    // Update current keys
                    if let Ok(new_key) = self.key_provider.get_primary_key().await {
                        *self.current_key.lock().await = new_key;
                    }
                    
                    if let Ok(backup) = self.key_provider.get_backup_key().await {
                        *self.backup_key.lock().await = backup;
                    }
                },
                Err(e) => {
                    error!("Failed to rotate API keys: {}", e);
                }
            }
        }
    }
}
```

### Data Privacy Protection

#### PII Detection and Redaction
```rust
use regex::Regex;
use once_cell::sync::Lazy;

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap()
});

static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b\d{3}-\d{3}-\d{4}\b|\b\(\d{3}\)\s*\d{3}-\d{4}\b").unwrap()
});

static SSN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap()
});

pub struct PIIRedactor {
    redaction_config: RedactionConfig,
}

#[derive(Debug, Clone)]
pub struct RedactionConfig {
    pub redact_emails: bool,
    pub redact_phone_numbers: bool,
    pub redact_ssns: bool,
    pub redaction_placeholder: String,
}

impl PIIRedactor {
    pub fn new(config: RedactionConfig) -> Self {
        Self {
            redaction_config: config,
        }
    }
    
    pub fn redact_text(&self, text: &str) -> String {
        let mut redacted = text.to_string();
        
        if self.redaction_config.redact_emails {
            redacted = EMAIL_REGEX.replace_all(&redacted, &self.redaction_config.redaction_placeholder).to_string();
        }
        
        if self.redaction_config.redact_phone_numbers {
            redacted = PHONE_REGEX.replace_all(&redacted, &self.redaction_config.redaction_placeholder).to_string();
        }
        
        if self.redaction_config.redact_ssns {
            redacted = SSN_REGEX.replace_all(&redacted, &self.redaction_config.redaction_placeholder).to_string();
        }
        
        redacted
    }
    
    pub fn detect_pii(&self, text: &str) -> Vec<PIIMatch> {
        let mut matches = Vec::new();
        
        for email_match in EMAIL_REGEX.find_iter(text) {
            matches.push(PIIMatch {
                pii_type: PIIType::Email,
                start: email_match.start(),
                end: email_match.end(),
                matched_text: email_match.as_str().to_string(),
            });
        }
        
        // Similar for phone numbers and SSNs...
        
        matches
    }
}

#[derive(Debug, Clone)]
pub struct PIIMatch {
    pub pii_type: PIIType,
    pub start: usize,
    pub end: usize,
    pub matched_text: String,
}

#[derive(Debug, Clone)]
pub enum PIIType {
    Email,
    PhoneNumber,
    SSN,
    CreditCard,
    Custom(String),
}
```

## Observability & Monitoring

### Metrics Collection

#### AI Service Metrics
```rust
use prometheus::{Counter, Histogram, Gauge, Registry};
use std::sync::Arc;

#[derive(Clone)]
pub struct AIMetrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub errors_total: Counter,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
    pub model_confidence: Histogram,
    pub active_requests: Gauge,
}

impl AIMetrics {
    pub fn new(registry: &Registry) -> Result<Self, Box<dyn std::error::Error>> {
        let requests_total = Counter::new(
            "ai_requests_total",
            "Total number of AI service requests"
        )?;
        registry.register(Box::new(requests_total.clone()))?;
        
        let request_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "ai_request_duration_seconds",
                "Duration of AI service requests"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0])
        )?;
        registry.register(Box::new(request_duration.clone()))?;
        
        let errors_total = Counter::new(
            "ai_errors_total",
            "Total number of AI service errors"
        )?;
        registry.register(Box::new(errors_total.clone()))?;
        
        let cache_hits = Counter::new(
            "ai_cache_hits_total",
            "Total number of cache hits"
        )?;
        registry.register(Box::new(cache_hits.clone()))?;
        
        let cache_misses = Counter::new(
            "ai_cache_misses_total",
            "Total number of cache misses"
        )?;
        registry.register(Box::new(cache_misses.clone()))?;
        
        let model_confidence = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "ai_model_confidence",
                "Confidence scores from AI models"
            ).buckets(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0])
        )?;
        registry.register(Box::new(model_confidence.clone()))?;
        
        let active_requests = Gauge::new(
            "ai_active_requests",
            "Number of currently active AI requests"
        )?;
        registry.register(Box::new(active_requests.clone()))?;
        
        Ok(Self {
            requests_total,
            request_duration,
            errors_total,
            cache_hits,
            cache_misses,
            model_confidence,
            active_requests,
        })
    }
}

pub struct MetricsMiddleware {
    metrics: Arc<AIMetrics>,
}

impl MetricsMiddleware {
    pub fn new(metrics: Arc<AIMetrics>) -> Self {
        Self { metrics }
    }
    
    pub async fn wrap_request<T, F, Fut>(
        &self,
        request_type: &str,
        operation: F,
    ) -> Result<T, AIError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AIError>>,
    {
        let _timer = self.metrics.request_duration.start_timer();
        self.metrics.active_requests.inc();
        self.metrics.requests_total.inc();
        
        let result = operation().await;
        
        self.metrics.active_requests.dec();
        
        match &result {
            Ok(_) => {
                // Request succeeded
            },
            Err(error) => {
                self.metrics.errors_total.inc();
                error!("AI request failed: {} - {}", request_type, error);
            }
        }
        
        result
    }
}
```

#### Sentiment Analysis Drift Detection
```rust
use std::collections::VecDeque;

pub struct SentimentDriftDetector {
    baseline_distribution: EmotionDistribution,
    recent_predictions: VecDeque<SentimentAnalysis>,
    window_size: usize,
    drift_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct EmotionDistribution {
    pub joy: f64,
    pub anger: f64,
    pub sadness: f64,
    pub fear: f64,
    pub disgust: f64,
    pub surprise: f64,
}

impl SentimentDriftDetector {
    pub fn new(baseline: EmotionDistribution, window_size: usize) -> Self {
        Self {
            baseline_distribution: baseline,
            recent_predictions: VecDeque::with_capacity(window_size),
            window_size,
            drift_threshold: 0.1, // 10% change triggers alert
        }
    }
    
    pub fn add_prediction(&mut self, prediction: SentimentAnalysis) {
        if self.recent_predictions.len() >= self.window_size {
            self.recent_predictions.pop_front();
        }
        self.recent_predictions.push_back(prediction);
    }
    
    pub fn check_for_drift(&self) -> Option<DriftAlert> {
        if self.recent_predictions.len() < self.window_size {
            return None; // Not enough data yet
        }
        
        let current_distribution = self.calculate_current_distribution();
        let drift_score = self.calculate_drift_score(&current_distribution);
        
        if drift_score > self.drift_threshold {
            Some(DriftAlert {
                drift_score,
                baseline_distribution: self.baseline_distribution.clone(),
                current_distribution,
                timestamp: chrono::Utc::now(),
            })
        } else {
            None
        }
    }
    
    fn calculate_current_distribution(&self) -> EmotionDistribution {
        let total_predictions = self.recent_predictions.len() as f64;
        let mut emotion_counts = std::collections::HashMap::new();
        
        for prediction in &self.recent_predictions {
            let emotion = &prediction.primary_emotion;
            *emotion_counts.entry(emotion.clone()).or_insert(0.0) += 1.0;
        }
        
        EmotionDistribution {
            joy: emotion_counts.get(&Emotion::Joy).unwrap_or(&0.0) / total_predictions,
            anger: emotion_counts.get(&Emotion::Anger).unwrap_or(&0.0) / total_predictions,
            sadness: emotion_counts.get(&Emotion::Sadness).unwrap_or(&0.0) / total_predictions,
            fear: emotion_counts.get(&Emotion::Fear).unwrap_or(&0.0) / total_predictions,
            disgust: emotion_counts.get(&Emotion::Disgust).unwrap_or(&0.0) / total_predictions,
            surprise: emotion_counts.get(&Emotion::Surprise).unwrap_or(&0.0) / total_predictions,
        }
    }
    
    fn calculate_drift_score(&self, current: &EmotionDistribution) -> f64 {
        // Calculate Kullback-Leibler divergence
        let baseline = &self.baseline_distribution;
        
        let kl_divergence = 
            self.kl_component(current.joy, baseline.joy) +
            self.kl_component(current.anger, baseline.anger) +
            self.kl_component(current.sadness, baseline.sadness) +
            self.kl_component(current.fear, baseline.fear) +
            self.kl_component(current.disgust, baseline.disgust) +
            self.kl_component(current.surprise, baseline.surprise);
        
        kl_divergence
    }
    
    fn kl_component(&self, p: f64, q: f64) -> f64 {
        if p == 0.0 {
            0.0
        } else if q == 0.0 {
            f64::INFINITY
        } else {
            p * (p / q).ln()
        }
    }
}

#[derive(Debug, Clone)]
pub struct DriftAlert {
    pub drift_score: f64,
    pub baseline_distribution: EmotionDistribution,
    pub current_distribution: EmotionDistribution,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

### Alerting System

```rust
use tokio::sync::mpsc;
use serde_json::json;

#[derive(Debug, Clone)]
pub enum Alert {
    HighErrorRate {
        service: String,
        error_rate: f64,
        threshold: f64,
    },
    HighLatency {
        service: String,
        p99_latency: f64,
        threshold: f64,
    },
    ModelDrift {
        drift_alert: DriftAlert,
    },
    ServiceDown {
        service: String,
        last_successful_check: chrono::DateTime<chrono::Utc>,
    },
}

pub struct AlertManager {
    alert_channels: Vec<Box<dyn AlertChannel>>,
    alert_queue: mpsc::UnboundedReceiver<Alert>,
    alert_sender: mpsc::UnboundedSender<Alert>,
}

#[async_trait]
pub trait AlertChannel: Send + Sync {
    async fn send_alert(&self, alert: &Alert) -> Result<(), AlertError>;
}

pub struct SlackAlertChannel {
    webhook_url: String,
    client: reqwest::Client,
}

#[async_trait]
impl AlertChannel for SlackAlertChannel {
    async fn send_alert(&self, alert: &Alert) -> Result<(), AlertError> {
        let message = match alert {
            Alert::HighErrorRate { service, error_rate, threshold } => {
                format!("🚨 High error rate detected in {}: {:.2}% (threshold: {:.2}%)", 
                       service, error_rate * 100.0, threshold * 100.0)
            },
            Alert::HighLatency { service, p99_latency, threshold } => {
                format!("⚠️ High latency detected in {}: {:.2}ms (threshold: {:.2}ms)", 
                       service, p99_latency, threshold)
            },
            Alert::ModelDrift { drift_alert } => {
                format!("📊 Model drift detected: score {:.3} at {}", 
                       drift_alert.drift_score, drift_alert.timestamp)
            },
            Alert::ServiceDown { service, last_successful_check } => {
                format!("💥 Service {} is down. Last successful check: {}", 
                       service, last_successful_check)
            },
        };
        
        let payload = json!({
            "text": message,
            "username": "Social Pulse Alert",
            "icon_emoji": ":warning:"
        });
        
        let response = self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AlertError::NetworkError(e.to_string()))?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(AlertError::DeliveryFailed(response.status().to_string()))
        }
    }
}

impl AlertManager {
    pub fn new() -> (Self, mpsc::UnboundedSender<Alert>) {
        let (alert_sender, alert_queue) = mpsc::unbounded_channel();
        let sender_clone = alert_sender.clone();
        
        let manager = Self {
            alert_channels: Vec::new(),
            alert_queue,
            alert_sender,
        };
        
        (manager, sender_clone)
    }
    
    pub fn add_channel(&mut self, channel: Box<dyn AlertChannel>) {
        self.alert_channels.push(channel);
    }
    
    pub async fn start_processing(&mut self) {
        info!("Starting alert manager...");
        
        while let Some(alert) = self.alert_queue.recv().await {
            info!("Processing alert: {:?}", alert);
            
            for channel in &self.alert_channels {
                if let Err(e) = channel.send_alert(&alert).await {
                    error!("Failed to send alert via channel: {}", e);
                }
            }
        }
    }
}
```

## Migration & Rollout Plan

### Phase 1: Infrastructure Setup (Week 1)

#### Environment Preparation
```bash
# Environment variables for Gen AI integration
export AI_PROVIDER=openai  # or azure, anthropic
export OPENAI_API_KEY=your_api_key_here
export OPENAI_MODEL=gpt-3.5-turbo
export AI_CACHE_ENABLED=true
export AI_CACHE_TTL_HOURS=24
export AI_TIMEOUT_SECONDS=5
export AI_MAX_RETRIES=3
export AI_FALLBACK_MODE=fail_open  # or fail_closed
```

#### Service Implementation
```rust
// Add to Cargo.toml
// [dependencies]
// openai = "1.0"
// async-openai = "0.17"
// tiktoken-rs = "0.5"

// Implementation scaffold
pub struct OpenAIService {
    client: async_openai::Client<OpenAIConfig>,
    model: String,
    config: OpenAIConfig,
}

impl OpenAIService {
    pub fn new(api_key: String, model: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = async_openai::Client::with_config(config.clone());
        
        Self {
            client,
            model,
            config,
        }
    }
}

#[async_trait]
impl AIService for OpenAIService {
    async fn analyze_sentiment(&self, text: &str, context: Option<&str>) -> Result<SentimentAnalysis, AIError> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(SENTIMENT_ANALYSIS_PROMPT)
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(format!("Text: {}\nContext: {}", text, context.unwrap_or("None")))
                    .build()?
                    .into(),
            ])
            .temperature(0.1)
            .max_tokens(500u16)
            .build()?;
        
        let response = self.client.chat().completions().create(request).await?;
        
        let content = response.choices[0].message.content.as_ref()
            .ok_or(AIError::EmptyResponse)?;
        
        self.parse_sentiment_response(content)
    }
    
    async fn moderate_content(&self, text: &str, context: Option<&str>) -> Result<ModerationResult, AIError> {
        // Similar implementation for content moderation
        todo!("Implement content moderation")
    }
}
```

### Phase 2: Parallel Testing (Week 2)

#### A/B Testing Framework
```rust
use rand::Rng;

pub struct ABTestManager {
    test_configs: HashMap<String, ABTestConfig>,
    user_assignments: HashMap<String, String>, // user_id -> variant
}

#[derive(Debug, Clone)]
pub struct ABTestConfig {
    pub test_name: String,
    pub variants: Vec<ABTestVariant>,
    pub traffic_allocation: f64, // 0.0 to 1.0
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ABTestVariant {
    pub name: String,
    pub weight: f64, // 0.0 to 1.0
    pub ai_service: AIServiceType,
}

#[derive(Debug, Clone)]
pub enum AIServiceType {
    CurrentPython,
    OpenAI,
    Azure,
    Anthropic,
}

impl ABTestManager {
    pub fn assign_user_to_test(&mut self, user_id: &str, test_name: &str) -> Option<String> {
        if let Some(config) = self.test_configs.get(test_name) {
            if !config.is_active() {
                return None;
            }
            
            // Check if user is already assigned
            if let Some(variant) = self.user_assignments.get(user_id) {
                return Some(variant.clone());
            }
            
            // Determine if user should be in the test
            let mut rng = rand::thread_rng();
            if rng.gen::<f64>() > config.traffic_allocation {
                return None; // User not in test
            }
            
            // Assign to variant based on weights
            let random_value = rng.gen::<f64>();
            let mut cumulative_weight = 0.0;
            
            for variant in &config.variants {
                cumulative_weight += variant.weight;
                if random_value <= cumulative_weight {
                    self.user_assignments.insert(user_id.to_string(), variant.name.clone());
                    return Some(variant.name.clone());
                }
            }
        }
        
        None
    }
    
    pub fn get_ai_service_for_user(&self, user_id: &str, test_name: &str) -> AIServiceType {
        if let Some(variant_name) = self.user_assignments.get(user_id) {
            if let Some(config) = self.test_configs.get(test_name) {
                for variant in &config.variants {
                    if &variant.name == variant_name {
                        return variant.ai_service.clone();
                    }
                }
            }
        }
        
        AIServiceType::CurrentPython // Default fallback
    }
}

impl ABTestConfig {
    fn is_active(&self) -> bool {
        let now = chrono::Utc::now();
        now >= self.start_date && now <= self.end_date
    }
}
```

#### Metrics Comparison
```rust
pub struct ABTestMetrics {
    pub variant_metrics: HashMap<String, VariantMetrics>,
}

#[derive(Debug, Clone)]
pub struct VariantMetrics {
    pub request_count: u64,
    pub avg_response_time: f64,
    pub error_rate: f64,
    pub accuracy_score: f64,
    pub user_satisfaction: f64,
    pub cost_per_request: f64,
}

impl ABTestMetrics {
    pub fn record_request(&mut self, variant: &str, response_time: f64, was_error: bool) {
        let metrics = self.variant_metrics.entry(variant.to_string())
            .or_insert_with(|| VariantMetrics::default());
        
        metrics.request_count += 1;
        
        // Update running average for response time
        let count = metrics.request_count as f64;
        metrics.avg_response_time = ((metrics.avg_response_time * (count - 1.0)) + response_time) / count;
        
        // Update error rate
        if was_error {
            metrics.error_rate = ((metrics.error_rate * (count - 1.0)) + 1.0) / count;
        } else {
            metrics.error_rate = (metrics.error_rate * (count - 1.0)) / count;
        }
    }
    
    pub fn generate_report(&self) -> ABTestReport {
        let mut comparisons = Vec::new();
        
        let variants: Vec<_> = self.variant_metrics.keys().collect();
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                let variant_a = variants[i];
                let variant_b = variants[j];
                
                let metrics_a = &self.variant_metrics[variant_a];
                let metrics_b = &self.variant_metrics[variant_b];
                
                let comparison = VariantComparison {
                    variant_a: variant_a.clone(),
                    variant_b: variant_b.clone(),
                    response_time_improvement: 
                        (metrics_a.avg_response_time - metrics_b.avg_response_time) / metrics_a.avg_response_time,
                    error_rate_improvement: 
                        (metrics_a.error_rate - metrics_b.error_rate) / metrics_a.error_rate,
                    accuracy_improvement: 
                        (metrics_b.accuracy_score - metrics_a.accuracy_score) / metrics_a.accuracy_score,
                    statistical_significance: self.calculate_significance(metrics_a, metrics_b),
                };
                
                comparisons.push(comparison);
            }
        }
        
        ABTestReport {
            test_duration: chrono::Duration::days(7), // Placeholder
            total_requests: self.variant_metrics.values().map(|m| m.request_count).sum(),
            variant_metrics: self.variant_metrics.clone(),
            comparisons,
            recommendation: self.make_recommendation(),
        }
    }
    
    fn calculate_significance(&self, metrics_a: &VariantMetrics, metrics_b: &VariantMetrics) -> f64 {
        // Simplified statistical significance calculation
        // In production, use proper statistical tests
        let sample_size_a = metrics_a.request_count as f64;
        let sample_size_b = metrics_b.request_count as f64;
        
        if sample_size_a < 100.0 || sample_size_b < 100.0 {
            return 0.0; // Insufficient data
        }
        
        // Placeholder calculation - implement proper t-test or chi-square test
        0.95 // Assume 95% confidence for now
    }
    
    fn make_recommendation(&self) -> String {
        // Logic to recommend the best variant
        let mut best_variant = None;
        let mut best_score = f64::NEG_INFINITY;
        
        for (variant_name, metrics) in &self.variant_metrics {
            // Composite score considering multiple factors
            let score = 
                (1.0 - metrics.error_rate) * 0.4 +
                metrics.accuracy_score * 0.3 +
                (1.0 / metrics.avg_response_time) * 0.2 +
                (1.0 / metrics.cost_per_request) * 0.1;
            
            if score > best_score {
                best_score = score;
                best_variant = Some(variant_name);
            }
        }
        
        match best_variant {
            Some(variant) => format!("Recommend migrating to variant: {}", variant),
            None => "Insufficient data to make recommendation".to_string(),
        }
    }
}
```

### Phase 3: Gradual Rollout (Week 3-4)

#### Feature Flag Implementation
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub genai_sentiment_analysis: FeatureFlag,
    pub genai_content_moderation: FeatureFlag,
    pub enhanced_explanations: FeatureFlag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub enabled: bool,
    pub rollout_percentage: f64, // 0.0 to 100.0
    pub user_allowlist: Vec<String>,
    pub user_blocklist: Vec<String>,
    pub conditions: Vec<RolloutCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RolloutCondition {
    pub condition_type: ConditionType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    UserProperty,
    RequestProperty,
    TimeWindow,
    ErrorRateThreshold,
}

pub struct FeatureFlagManager {
    flags: Arc<RwLock<FeatureFlags>>,
    config_source: Box<dyn ConfigSource>,
}

#[async_trait]
pub trait ConfigSource: Send + Sync {
    async fn load_config(&self) -> Result<FeatureFlags, ConfigError>;
    async fn watch_for_changes(&self) -> Result<tokio::sync::mpsc::Receiver<FeatureFlags>, ConfigError>;
}

impl FeatureFlagManager {
    pub async fn new(config_source: Box<dyn ConfigSource>) -> Result<Self, ConfigError> {
        let initial_flags = config_source.load_config().await?;
        
        let manager = Self {
            flags: Arc::new(RwLock::new(initial_flags)),
            config_source,
        };
        
        // Start watching for config changes
        manager.start_config_watcher().await?;
        
        Ok(manager)
    }
    
    pub async fn is_enabled(&self, flag_name: &str, user_id: &str, context: &RequestContext) -> bool {
        let flags = self.flags.read().await;
        
        let flag = match flag_name {
            "genai_sentiment_analysis" => &flags.genai_sentiment_analysis,
            "genai_content_moderation" => &flags.genai_content_moderation,
            "enhanced_explanations" => &flags.enhanced_explanations,
            _ => return false,
        };
        
        if !flag.enabled {
            return false;
        }
        
        // Check blocklist
        if flag.user_blocklist.contains(&user_id.to_string()) {
            return false;
        }
        
        // Check allowlist
        if !flag.user_allowlist.is_empty() && flag.user_allowlist.contains(&user_id.to_string()) {
            return true;
        }
        
        // Check rollout percentage
        let user_hash = self.hash_user_id(user_id);
        let user_percentage = (user_hash % 100) as f64;
        
        if user_percentage >= flag.rollout_percentage {
            return false;
        }
        
        // Check additional conditions
        for condition in &flag.conditions {
            if !self.evaluate_condition(condition, user_id, context).await {
                return false;
            }
        }
        
        true
    }
    
    fn hash_user_id(&self, user_id: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        hasher.finish()
    }
    
    async fn evaluate_condition(&self, condition: &RolloutCondition, user_id: &str, context: &RequestContext) -> bool {
        match condition.condition_type {
            ConditionType::UserProperty => {
                // Check user properties
                context.user_properties.get(&condition.value).is_some()
            },
            ConditionType::RequestProperty => {
                // Check request properties
                context.request_properties.get(&condition.value).is_some()
            },
            ConditionType::TimeWindow => {
                // Check if current time is within specified window
                // Parse condition.value as time range
                true // Placeholder
            },
            ConditionType::ErrorRateThreshold => {
                // Check if current error rate is below threshold
                let threshold: f64 = condition.value.parse().unwrap_or(1.0);
                // Get current error rate from metrics
                true // Placeholder
            },
        }
    }
    
    async fn start_config_watcher(&self) -> Result<(), ConfigError> {
        let mut change_receiver = self.config_source.watch_for_changes().await?;
        let flags_clone = Arc::clone(&self.flags);
        
        tokio::spawn(async move {
            while let Some(new_flags) = change_receiver.recv().await {
                info!("Updating feature flags from config source");
                *flags_clone.write().await = new_flags;
            }
        });
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub user_properties: HashMap<String, String>,
    pub request_properties: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

#### Rollout Monitoring
```rust
pub struct RolloutMonitor {
    metrics_collector: Arc<AIMetrics>,
    alert_sender: mpsc::UnboundedSender<Alert>,
    rollout_state: Arc<RwLock<RolloutState>>,
}

#[derive(Debug, Clone)]
pub struct RolloutState {
    pub current_percentage: f64,
    pub target_percentage: f64,
    pub rollout_start_time: chrono::DateTime<chrono::Utc>,
    pub last_increment_time: chrono::DateTime<chrono::Utc>,
    pub success_metrics: RolloutMetrics,
    pub rollback_triggers: Vec<RollbackTrigger>,
}

#[derive(Debug, Clone)]
pub struct RolloutMetrics {
    pub error_rate: f64,
    pub avg_response_time: f64,
    pub success_rate: f64,
    pub user_satisfaction: f64,
}

#[derive(Debug, Clone)]
pub struct RollbackTrigger {
    pub trigger_type: TriggerType,
    pub threshold: f64,
    pub duration: chrono::Duration,
}

#[derive(Debug, Clone)]
pub enum TriggerType {
    ErrorRateSpike,
    LatencyIncrease,
    SuccessRateDropp,
    UserComplaintSpike,
}

impl RolloutMonitor {
    pub async fn start_monitoring(&self) {
        let mut interval = tokio::time::interval(Duration::from_minutes(5));
        
        loop {
            interval.tick().await;
            
            let current_metrics = self.collect_current_metrics().await;
            let rollout_state = self.rollout_state.read().await.clone();
            
            // Check for rollback triggers
            if self.should_rollback(&current_metrics, &rollout_state).await {
                warn!("Rollback triggered due to poor metrics");
                self.initiate_rollback().await;
                continue;
            }
            
            // Check if we should continue rollout
            if self.should_continue_rollout(&current_metrics, &rollout_state).await {
                self.increment_rollout().await;
            }
        }
    }
    
    async fn collect_current_metrics(&self) -> RolloutMetrics {
        // Collect metrics from the metrics collector
        // This is a simplified version
        RolloutMetrics {
            error_rate: 0.01, // 1%
            avg_response_time: 150.0, // 150ms
            success_rate: 0.99, // 99%
            user_satisfaction: 0.85, // 85%
        }
    }
    
    async fn should_rollback(&self, current: &RolloutMetrics, state: &RolloutState) -> bool {
        for trigger in &state.rollback_triggers {
            match trigger.trigger_type {
                TriggerType::ErrorRateSpike => {
                    if current.error_rate > trigger.threshold {
                        return true;
                    }
                },
                TriggerType::LatencyIncrease => {
                    if current.avg_response_time > trigger.threshold {
                        return true;
                    }
                },
                TriggerType::SuccessRateDropp => {
                    if current.success_rate < trigger.threshold {
                        return true;
                    }
                },
                TriggerType::UserComplaintSpike => {
                    if current.user_satisfaction < trigger.threshold {
                        return true;
                    }
                },
            }
        }
        
        false
    }
    
    async fn should_continue_rollout(&self, current: &RolloutMetrics, state: &RolloutState) -> bool {
        // Check if metrics are healthy and enough time has passed
        let time_since_last_increment = chrono::Utc::now() - state.last_increment_time;
        let min_wait_time = chrono::Duration::hours(2); // Wait at least 2 hours between increments
        
        if time_since_last_increment < min_wait_time {
            return false;
        }
        
        // Check if metrics are healthy
        current.error_rate < 0.02 && // Less than 2% error rate
        current.avg_response_time < 300.0 && // Less than 300ms response time
        current.success_rate > 0.98 && // Greater than 98% success rate
        current.user_satisfaction > 0.8 // Greater than 80% satisfaction
    }
    
    async fn increment_rollout(&self) {
        let mut state = self.rollout_state.write().await;
        
        if state.current_percentage >= state.target_percentage {
            info!("Rollout complete at {}%", state.current_percentage);
            return;
        }
        
        // Increment by 10% at a time
        let increment = 10.0;
        state.current_percentage = (state.current_percentage + increment).min(state.target_percentage);
        state.last_increment_time = chrono::Utc::now();
        
        info!("Incremented rollout to {}%", state.current_percentage);
        
        // Update feature flag configuration
        // This would typically involve calling your config management system
    }
    
    async fn initiate_rollback(&self) {
        error!("Initiating automatic rollback due to poor metrics");
        
        let mut state = self.rollout_state.write().await;
        state.current_percentage = 0.0;
        
        // Send alert
        let alert = Alert::ServiceDown {
            service: "GenAI Rollout".to_string(),
            last_successful_check: chrono::Utc::now(),
        };
        
        if let Err(e) = self.alert_sender.send(alert) {
            error!("Failed to send rollback alert: {}", e);
        }
        
        // Update feature flag to disable Gen AI
        // This would typically involve calling your config management system
    }
}
```

### Phase 4: Full Migration (Week 5-6)

#### Migration Completion Checklist

```rust
pub struct MigrationValidator {
    ai_service: Box<dyn AIService>,
    test_cases: Vec<TestCase>,
    success_criteria: SuccessCriteria,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub input_text: String,
    pub expected_sentiment: Option<Emotion>,
    pub expected_moderation: Option<bool>,
    pub context: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SuccessCriteria {
    pub min_accuracy: f64,
    pub max_error_rate: f64,
    pub max_response_time: f64,
    pub min_uptime: f64,
}

impl MigrationValidator {
    pub async fn validate_migration(&self) -> Result<MigrationReport, ValidationError> {
        let mut results = Vec::new();
        
        info!("Starting migration validation with {} test cases", self.test_cases.len());
        
        for (i, test_case) in self.test_cases.iter().enumerate() {
            info!("Running test case {}/{}", i + 1, self.test_cases.len());
            
            let start_time = std::time::Instant::now();
            
            // Test sentiment analysis
            let sentiment_result = self.ai_service
                .analyze_sentiment(&test_case.input_text, test_case.context.as_deref())
                .await;
            
            let sentiment_duration = start_time.elapsed();
            
            // Test content moderation
            let moderation_result = self.ai_service
                .moderate_content(&test_case.input_text, test_case.context.as_deref())
                .await;
            
            let total_duration = start_time.elapsed();
            
            let test_result = TestResult {
                test_case: test_case.clone(),
                sentiment_result,
                moderation_result,
                response_time: total_duration,
                passed: self.evaluate_test_result(test_case, &sentiment_result, &moderation_result),
            };
            
            results.push(test_result);
        }
        
        let report = self.generate_migration_report(results);
        
        if report.meets_success_criteria(&self.success_criteria) {
            info!("✅ Migration validation passed!");
        } else {
            warn!("❌ Migration validation failed!");
        }
        
        Ok(report)
    }
    
    fn evaluate_test_result(
        &self,
        test_case: &TestCase,
        sentiment_result: &Result<SentimentAnalysis, AIError>,
        moderation_result: &Result<ModerationResult, AIError>,
    ) -> bool {
        // Check sentiment accuracy
        if let (Some(expected), Ok(actual)) = (&test_case.expected_sentiment, sentiment_result) {
            if &actual.primary_emotion != expected {
                return false;
            }
        }
        
        // Check moderation accuracy
        if let (Some(expected), Ok(actual)) = (&test_case.expected_moderation, moderation_result) {
            if &actual.is_blocked != expected {
                return false;
            }
        }
        
        // Check for errors
        if sentiment_result.is_err() || moderation_result.is_err() {
            return false;
        }
        
        true
    }
    
    fn generate_migration_report(&self, results: Vec<TestResult>) -> MigrationReport {
        let total_tests = results.len();
        let passed_tests = results.iter().filter(|r| r.passed).count();
        let accuracy = passed_tests as f64 / total_tests as f64;
        
        let avg_response_time = results.iter()
            .map(|r| r.response_time.as_millis() as f64)
            .sum::<f64>() / total_tests as f64;
        
        let error_count = results.iter()
            .filter(|r| r.sentiment_result.is_err() || r.moderation_result.is_err())
            .count();
        let error_rate = error_count as f64 / total_tests as f64;
        
        MigrationReport {
            total_tests,
            passed_tests,
            accuracy,
            error_rate,
            avg_response_time,
            detailed_results: results,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub accuracy: f64,
    pub error_rate: f64,
    pub avg_response_time: f64,
    pub detailed_results: Vec<TestResult>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MigrationReport {
    pub fn meets_success_criteria(&self, criteria: &SuccessCriteria) -> bool {
        self.accuracy >= criteria.min_accuracy &&
        self.error_rate <= criteria.max_error_rate &&
        self.avg_response_time <= criteria.max_response_time
    }
    
    pub fn generate_summary(&self) -> String {
        format!(
            "Migration Validation Report\n" +
            "========================\n" +
            "Total Tests: {}\n" +
            "Passed Tests: {}\n" +
            "Accuracy: {:.2}%\n" +
            "Error Rate: {:.2}%\n" +
            "Avg Response Time: {:.2}ms\n" +
            "Timestamp: {}\n",
            self.total_tests,
            self.passed_tests,
            self.accuracy * 100.0,
            self.error_rate * 100.0,
            self.avg_response_time,
            self.timestamp
        )
    }
}
```

## Appendix: Code Examples

### Complete OpenAI Integration Example

```rust
use async_openai::{Client, types::*};
use serde_json::Value;
use std::time::Duration;

pub struct OpenAIGenAIService {
    client: Client<OpenAIConfig>,
    model: String,
    timeout: Duration,
}

impl OpenAIGenAIService {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);
        
        Self {
            client,
            model: model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            timeout: Duration::from_secs(10),
        }
    }
    
    async fn call_openai_with_prompt(&self, prompt: &str, user_input: &str) -> Result<String, AIError> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(prompt)
                    .build()
                    .map_err(|e| AIError::RequestBuilderError(e.to_string()))?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(user_input)
                    .build()
                    .map_err(|e| AIError::RequestBuilderError(e.to_string()))?
                    .into(),
            ])
            .temperature(0.1)
            .max_tokens(1000u16)
            .build()
            .map_err(|e| AIError::RequestBuilderError(e.to_string()))?;
        
        let response = tokio::time::timeout(
            self.timeout,
            self.client.chat().completions().create(request)
        )
        .await
        .map_err(|_| AIError::Timeout)??
        .map_err(|e| AIError::APIError(e.to_string()))?;
        
        response.choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or(AIError::EmptyResponse)
    }
}

#[async_trait]
impl AIService for OpenAIGenAIService {
    async fn analyze_sentiment(&self, text: &str, context: Option<&str>) -> Result<SentimentAnalysis, AIError> {
        let context_str = context.unwrap_or("None");
        let user_input = format!("Text: {}\nContext: {}", text, context_str);
        
        let response = self.call_openai_with_prompt(SENTIMENT_ANALYSIS_PROMPT, &user_input).await?;
        
        // Parse JSON response
        let parsed: Value = serde_json::from_str(&response)
            .map_err(|e| AIError::ParseError(format!("Failed to parse sentiment response: {}", e)))?;
        
        let primary_emotion = parsed["primary_emotion"]
            .as_str()
            .ok_or(AIError::ParseError("Missing primary_emotion".to_string()))?;
        
        let confidence = parsed["confidence"]
            .as_f64()
            .ok_or(AIError::ParseError("Missing confidence".to_string()))?;
        
        let explanation = parsed["explanation"]
            .as_str()
            .map(|s| s.to_string());
        
        // Parse emotion scores
        let emotion_scores = parsed["emotion_scores"]
            .as_object()
            .ok_or(AIError::ParseError("Missing emotion_scores".to_string()))?
            .iter()
            .filter_map(|(k, v)| {
                v.as_f64().map(|score| (Emotion::from_str(k).unwrap_or(Emotion::Neutral), score))
            })
            .collect();
        
        Ok(SentimentAnalysis {
            primary_emotion: Emotion::from_str(primary_emotion).unwrap_or(Emotion::Neutral),
            emotion_scores,
            confidence,
            explanation,
            context_factors: Vec::new(), // Could be extracted from response
            cultural_notes: None, // Could be extracted from response
        })
    }
    
    async fn moderate_content(&self, text: &str, context: Option<&str>) -> Result<ModerationResult, AIError> {
        let context_str = context.unwrap_or("None");
        let user_input = format!("Content: {}\nContext: {}", text, context_str);
        
        let response = self.call_openai_with_prompt(MODERATION_PROMPT, &user_input).await?;
        
        // Parse JSON response
        let parsed: Value = serde_json::from_str(&response)
            .map_err(|e| AIError::ParseError(format!("Failed to parse moderation response: {}", e)))?;
        
        let is_blocked = parsed["is_blocked"]
            .as_bool()
            .ok_or(AIError::ParseError("Missing is_blocked".to_string()))?;
        
        let risk_level_str = parsed["risk_level"]
            .as_str()
            .ok_or(AIError::ParseError("Missing risk_level".to_string()))?;
        
        let confidence = parsed["confidence"]
            .as_f64()
            .ok_or(AIError::ParseError("Missing confidence".to_string()))?;
        
        let explanation = parsed["explanation"]
            .as_str()
            .ok_or(AIError::ParseError("Missing explanation".to_string()))?
            .to_string();
        
        let violation_categories = parsed["violation_categories"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| ViolationCategory::from_str(s)))
                    .collect()
            })
            .unwrap_or_default();
        
        Ok(ModerationResult {
            is_blocked,
            risk_level: RiskLevel::from_str(risk_level_str),
            violation_categories,
            confidence,
            explanation,
            suggested_action: if is_blocked { ModerationAction::ContentRemoval { reason: explanation.clone() } } else { ModerationAction::Allow },
            appeal_guidance: Some("If you believe this decision is incorrect, please contact support.".to_string()),
        })
    }
    
    async fn explain_decision(&self, text: &str, decision: &str) -> Result<String, AIError> {
        let prompt = format!(
            "Explain why the following content moderation decision was made:\n\n" +
            "Content: {}\n" +
            "Decision: {}\n\n" +
            "Provide a clear, helpful explanation that helps the user understand the decision.",
            text, decision
        );
        
        self.call_openai_with_prompt(&prompt, "").await
    }
}
```

### Environment Configuration Template

```bash
# .env.example

# AI Provider Configuration
AI_PROVIDER=openai  # Options: openai, azure, anthropic, local

# OpenAI Configuration
OPENAI_API_KEY=sk-your-openai-api-key-here
OPENAI_MODEL=gpt-3.5-turbo
OPENAI_ENDPOINT=https://api.openai.com/v1  # Optional: custom endpoint

# Azure OpenAI Configuration (if using Azure)
AZURE_OPENAI_API_KEY=your-azure-api-key-here
AZURE_OPENAI_ENDPOINT=https://your-resource.openai.azure.com/
AZURE_OPENAI_MODEL=gpt-35-turbo
AZURE_REGION=eastus

# AI Service Configuration
AI_TIMEOUT_SECONDS=10
AI_MAX_RETRIES=3
AI_FALLBACK_MODE=fail_open  # Options: fail_open, fail_closed, use_fallback

# Caching Configuration
AI_CACHE_ENABLED=true
AI_CACHE_TTL_HOURS=24
AI_CACHE_MAX_SIZE=10000

# Feature Flags
GENAI_SENTIMENT_ANALYSIS_ENABLED=true
GENAI_CONTENT_MODERATION_ENABLED=true
GENAI_ENHANCED_EXPLANATIONS_ENABLED=false

# Rollout Configuration
GENAI_ROLLOUT_PERCENTAGE=50.0
GENAI_AB_TEST_ENABLED=true

# Monitoring Configuration
METRICS_ENABLED=true
ALERT_WEBHOOK_URL=https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK

# Safety Configuration
PII_REDACTION_ENABLED=true
BIAS_DETECTION_ENABLED=true
CONTENT_LOGGING_ENABLED=false  # Set to false in production for privacy
```

---

*This guide provides a comprehensive framework for adopting Generative AI in your emotion detection and content moderation systems. The phased approach ensures safe, measurable migration while maintaining system reliability and user trust.*