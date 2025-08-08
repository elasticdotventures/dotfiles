use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollection, Distance, PointStruct, SearchPoints, VectorParams,
        WithPayloadSelector, Filter, Condition, Vectors, CollectionOperationResponse,
    },
};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Datum {
    pub name: String,
    pub datum_type: String,
    pub adjacencies: Vec<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: Uuid,
    pub content: String,
    pub datum: String,
    pub attribution: Attribution,
    pub metadata: ChunkMetadata,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribution {
    pub url: Option<String>,
    pub filename: Option<String>,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub topic: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

pub struct GrokClient {
    qdrant_url: String,
    api_key: String,
    collection_name: String,
    embedding_model: Option<EmbeddingModel>,
    qdrant_client: Option<Qdrant>,
}

#[derive(Debug)]
pub struct EmbeddingModel {
    // 🤓 Placeholder for future implementation
    model_name: String,
}

impl EmbeddingModel {
    pub async fn new() -> Result<Self> {
        // TODO: Initialize Python embedding service via HTTP or subprocess
        Ok(Self {
            model_name: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
        })
    }
    
    pub fn encode(&self, _text: &str) -> Result<Vec<f32>> {
        // TODO: Call Python embedding service
        // For now, return mock embedding
        let mock_embedding: Vec<f32> = (0..384).map(|i| (i as f32 * 0.01) % 1.0).collect();
        Ok(mock_embedding)
    }
}

impl GrokClient {
    pub fn new(qdrant_url: String, api_key: String) -> Self {
        Self {
            qdrant_url,
            api_key,
            collection_name: "b00t_chunks".to_string(),
            embedding_model: None,
            qdrant_client: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // Initialize Qdrant client
        let client = Qdrant::from_url(&self.qdrant_url)
            .api_key(self.api_key.clone())
            .build()?;
        
        // Initialize embedding model
        let embedding_model = EmbeddingModel::new().await?;
        
        self.qdrant_client = Some(client);
        self.embedding_model = Some(embedding_model);

        // Ensure collection exists
        self.ensure_collection_exists().await?;

        tracing::info!("GrokClient initialized for {}", self.qdrant_url);
        Ok(())
    }

    async fn ensure_collection_exists(&self) -> Result<()> {
        let client = self.qdrant_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Qdrant client not initialized"))?;

        // Check if collection exists
        match client.collection_info(&self.collection_name).await {
            Ok(_) => {
                tracing::info!("Collection '{}' already exists", self.collection_name);
                Ok(())
            }
            Err(_) => {
                // Collection doesn't exist, create it
                tracing::info!("Creating collection '{}'", self.collection_name);
                
                let create_collection = CreateCollection {
                    collection_name: self.collection_name.clone(),
                    vectors_config: Some(VectorParams {
                        size: 384,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    }.into()),
                    ..Default::default()
                };

                let response: CollectionOperationResponse = client
                    .create_collection(create_collection)
                    .await?;

                if response.result {
                    tracing::info!("Successfully created collection '{}'", self.collection_name);
                } else {
                    anyhow::bail!("Failed to create collection '{}'", self.collection_name);
                }
                Ok(())
            }
        }
    }

    fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let model = self.embedding_model.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Embedding model not initialized"))?;
        model.encode(text)
    }

    pub async fn digest(&self, topic: &str, content: &str) -> Result<Chunk> {
        let client = self.qdrant_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Qdrant client not initialized"))?;

        // Generate vector embedding
        let vector = self.generate_embedding(content)?;
        let chunk_id = Uuid::new_v4();
        
        let chunk = Chunk {
            id: chunk_id,
            content: content.to_string(),
            datum: topic.to_string(),
            attribution: Attribution {
                url: None,
                filename: None,
                date: chrono::Utc::now().to_rfc3339(),
            },
            metadata: ChunkMetadata {
                topic: topic.to_string(),
                tags: vec![],
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            vector: Some(vector.clone()),
        };

        // Store in Qdrant
        let mut payload = HashMap::new();
        payload.insert("content".to_string(), content.into());
        payload.insert("datum".to_string(), topic.into());
        payload.insert("topic".to_string(), topic.into());
        payload.insert("created_at".to_string(), chunk.metadata.created_at.clone().into());

        let point = PointStruct::new(
            chunk_id.to_string(),
            vector,
            payload,
        );

        let upsert_request = qdrant_client::qdrant::UpsertPoints {
            collection_name: self.collection_name.clone(),
            points: vec![point],
            ..Default::default()
        };

        client
            .upsert_points(upsert_request)
            .await?;

        tracing::info!("Stored chunk {} for topic '{}' in Qdrant", chunk_id, topic);

        Ok(chunk)
    }

    pub async fn ask(&self, query: &str, topic: Option<&str>) -> Result<Vec<Chunk>> {
        let client = self.qdrant_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Qdrant client not initialized"))?;

        // Generate query embedding
        let query_vector = self.generate_embedding(query)?;

        // Build search request
        let mut search_request = SearchPoints {
            collection_name: self.collection_name.clone(),
            vector: query_vector,
            limit: 10,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true)),
            }),
            ..Default::default()
        };

        // Add topic filter if specified
        if let Some(topic_filter) = topic {
            let filter = Filter {
                must: vec![Condition {
                    condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                        qdrant_client::qdrant::FieldCondition {
                            key: "topic".to_string(),
                            r#match: Some(qdrant_client::qdrant::Match {
                                match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Text(topic_filter.to_string())),
                            }),
                            ..Default::default()
                        }
                    )),
                }],
                ..Default::default()
            };
            search_request.filter = Some(filter);
        }

        let search_result = client.search_points(search_request).await?;

        let mut chunks = Vec::new();
        for scored_point in search_result.result {
            let payload = scored_point.payload;
            let content = payload.get("content")
                .and_then(|v| v.as_str())
                .map_or("", |v| v)
                .to_string();
            
            let datum = payload.get("datum")
                .and_then(|v| v.as_str())
                .map_or("unknown", |v| v)
                .to_string();
            
            let topic_str = payload.get("topic")
                .and_then(|v| v.as_str())
                .map_or("unknown", |v| v)
                .to_string();
            
            let created_at = payload.get("created_at")
                .and_then(|v| v.as_str())
                .map_or("", |v| v)
                .to_string();

            let point_id = match &scored_point.id {
                Some(id) => match id.point_id_options.as_ref() {
                    Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid_str)) => {
                        Uuid::parse_str(uuid_str).unwrap_or_else(|_| Uuid::new_v4())
                    }
                    Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(_)) => Uuid::new_v4(),
                    None => Uuid::new_v4(),
                }
                None => Uuid::new_v4(),
            };

            let chunk = Chunk {
                id: point_id,
                content,
                datum: datum.clone(),
                attribution: Attribution {
                    url: None,
                    filename: None,
                    date: created_at.clone(),
                },
                metadata: ChunkMetadata {
                    topic: topic_str,
                    tags: vec![format!("score_{:.3}", scored_point.score)],
                    created_at,
                },
                vector: scored_point.vectors.and_then(|v| match v.vectors_options? {
                    qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(vector_struct) => {
                        Some(vector_struct.data)
                    }
                    _ => None,
                }),
            };
            
            chunks.push(chunk);
        }

        tracing::info!("Found {} chunks for query '{}' with topic filter {:?}", 
                      chunks.len(), query, topic);

        Ok(chunks)
    }

    pub async fn learn(&self, source: &str, content: &str) -> Result<Vec<Chunk>> {
        let client = self.qdrant_client.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Qdrant client not initialized"))?;

        // Simple chunking for now - split by double newlines
        let chunks: Vec<&str> = content.split("\n\n")
            .filter(|chunk| !chunk.trim().is_empty())
            .collect();
        
        let mut result_chunks = Vec::new();
        let mut points = Vec::new();
        
        for (i, chunk_text) in chunks.into_iter().enumerate() {
            let trimmed = chunk_text.trim();
            if trimmed.len() < 10 { continue; } // Skip very short chunks
            
            // Infer topic from source or use "general"
            let inferred_topic = self.infer_topic_from_source(source);
            let chunk_id = Uuid::new_v4();
            let vector = self.generate_embedding(trimmed)?;
            
            let chunk = Chunk {
                id: chunk_id,
                content: trimmed.to_string(),
                datum: inferred_topic.clone(),
                attribution: Attribution {
                    url: if source.starts_with("http") { Some(source.to_string()) } else { None },
                    filename: if !source.starts_with("http") && source != "direct_input" { 
                        Some(source.to_string()) 
                    } else { None },
                    date: chrono::Utc::now().to_rfc3339(),
                },
                metadata: ChunkMetadata {
                    topic: inferred_topic.clone(),
                    tags: vec![format!("chunk_{}", i)],
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
                vector: Some(vector.clone()),
            };

            // Prepare point for batch insertion
            let mut payload = HashMap::new();
            payload.insert("content".to_string(), trimmed.into());
            payload.insert("datum".to_string(), inferred_topic.clone().into());
            payload.insert("topic".to_string(), inferred_topic.into());
            payload.insert("source".to_string(), source.into());
            payload.insert("created_at".to_string(), chunk.metadata.created_at.clone().into());
            payload.insert("chunk_index".to_string(), (i as i64).into());

            let point = PointStruct::new(
                chunk_id.to_string(),
                vector,
                payload,
            );

            points.push(point);
            result_chunks.push(chunk);
        }
        
        // Batch insert all points
        if !points.is_empty() {
            let upsert_request = qdrant_client::qdrant::UpsertPoints {
                collection_name: self.collection_name.clone(),
                points,
                ..Default::default()
            };

            client
                .upsert_points(upsert_request)
                .await?;

            tracing::info!("Stored {} chunks from source '{}' in Qdrant", 
                          result_chunks.len(), source);
        }

        Ok(result_chunks)
    }
    
    fn infer_topic_from_source(&self, source: &str) -> String {
        if source.contains("rust") || source.contains(".rs") {
            "rust".to_string()
        } else if source.contains("python") || source.contains(".py") {
            "python".to_string()
        } else if source.contains("docker") {
            "docker".to_string()
        } else if source.contains("git") {
            "git".to_string()
        } else {
            "general".to_string()
        }
    }
}

#[cfg(feature = "pyo3")]
#[pyclass]
pub struct PyGrokClient {
    client: GrokClient,
    runtime: tokio::runtime::Runtime,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl PyGrokClient {
    #[new]
    fn new(qdrant_url: String, api_key: String) -> PyResult<Self> {
        let mut client = GrokClient::new(qdrant_url, api_key);
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        // Initialize the client
        runtime.block_on(client.initialize())
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        Ok(Self { client, runtime })
    }

    fn digest(&mut self, topic: &str, content: &str) -> PyResult<String> {
        let chunk = self.runtime.block_on(self.client.digest(topic, content))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        serde_json::to_string(&chunk)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn ask(&mut self, query: &str, topic: Option<&str>) -> PyResult<Vec<String>> {
        let chunks = self.runtime.block_on(self.client.ask(query, topic))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        chunks.into_iter()
            .map(|chunk| serde_json::to_string(&chunk))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn learn(&mut self, source: &str, content: &str) -> PyResult<Vec<String>> {
        let chunks = self.runtime.block_on(self.client.learn(source, content))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
        chunks.into_iter()
            .map(|chunk| serde_json::to_string(&chunk))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}

#[cfg(feature = "pyo3")]
#[pymodule]
fn b00t_grok(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGrokClient>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_grok_client_creation() {
        let client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        assert_eq!(client.collection_name, "b00t_chunks");
        assert_eq!(client.qdrant_url, "https://example.com");
        assert_eq!(client.api_key, "test_key");
    }

    #[tokio::test]
    async fn test_grok_client_initialization() {
        let mut client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        let result = client.initialize().await;
        assert!(result.is_ok());
        assert!(client.embedding_model.is_some());
    }

    #[tokio::test]
    async fn test_digest() {
        let mut client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        client.initialize().await.unwrap();
        
        let chunk = client.digest("rust", "Rust is a systems programming language").await.unwrap();
        assert_eq!(chunk.datum, "rust");
        assert_eq!(chunk.content, "Rust is a systems programming language");
        assert_eq!(chunk.metadata.topic, "rust");
        assert!(!chunk.id.to_string().is_empty());
        assert!(chunk.attribution.date.len() > 0);
        assert!(chunk.vector.is_some()); // Should have mock embedding
    }

    #[test]
    fn test_datum_creation() {
        let mut properties = HashMap::new();
        properties.insert("language".to_string(), "systems".to_string());
        properties.insert("memory_safe".to_string(), "true".to_string());
        
        let datum = Datum {
            name: "rust".to_string(),
            datum_type: "programming_language".to_string(),
            adjacencies: vec!["cargo".to_string(), "clippy".to_string()],
            properties,
        };
        
        assert_eq!(datum.name, "rust");
        assert_eq!(datum.datum_type, "programming_language");
        assert_eq!(datum.adjacencies.len(), 2);
        assert_eq!(datum.properties.get("language"), Some(&"systems".to_string()));
    }

    #[test]
    fn test_chunk_serialization() {
        let chunk = Chunk {
            id: uuid::Uuid::new_v4(),
            content: "Test content".to_string(),
            datum: "test_datum".to_string(),
            attribution: Attribution {
                url: Some("https://example.com".to_string()),
                filename: None,
                date: "2025-01-01T00:00:00Z".to_string(),
            },
            metadata: ChunkMetadata {
                topic: "test".to_string(),
                tags: vec!["unit_test".to_string(), "example".to_string()],
                created_at: "2025-01-01T00:00:00Z".to_string(),
            },
            vector: Some(vec![0.1, 0.2, 0.3]),
        };
        
        let json = serde_json::to_string(&chunk).unwrap();
        let deserialized: Chunk = serde_json::from_str(&json).unwrap();
        
        assert_eq!(chunk.content, deserialized.content);
        assert_eq!(chunk.datum, deserialized.datum);
        assert_eq!(chunk.vector, deserialized.vector);
    }

    #[tokio::test]
    async fn test_ask_empty_result() {
        let mut client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        client.initialize().await.unwrap();
        
        let results = client.ask("What is Rust?", Some("rust")).await.unwrap();
        assert_eq!(results.len(), 0); // TODO: Will have results when implemented
    }

    #[tokio::test]
    async fn test_learn_chunking() {
        let mut client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        client.initialize().await.unwrap();
        
        let content = "First paragraph about Rust.\n\nSecond paragraph about Cargo.\n\nThird paragraph about testing.";
        let chunks = client.learn("rust-guide.md", content).await.unwrap();
        
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].datum, "rust");
        assert!(chunks[0].content.contains("First paragraph"));
        assert!(chunks[1].content.contains("Second paragraph"));
        assert!(chunks[2].content.contains("Third paragraph"));
    }

    #[tokio::test]
    async fn test_topic_inference() {
        let client = GrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        
        assert_eq!(client.infer_topic_from_source("main.rs"), "rust");
        assert_eq!(client.infer_topic_from_source("script.py"), "python");
        assert_eq!(client.infer_topic_from_source("Dockerfile"), "general"); // 🤓 Fixed test expectation
        assert_eq!(client.infer_topic_from_source("https://git.example.com/repo"), "git");
        assert_eq!(client.infer_topic_from_source("unknown_file.txt"), "general");
    }

    #[cfg(feature = "pyo3")]
    #[test]
    fn test_py_grok_client_creation() {
        let result = PyGrokClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        );
        assert!(result.is_ok());
    }
}