// src/topology/llm_substrate/qwen35.rs
//
// Implementazione completa Qwen3.5 in Candle
// Architettura: Linear Attention + Full Attention ibrida

use anyhow::{Context, Result};
use candle_core::{Device, DType, Tensor, IndexOp};
use candle_nn::{VarBuilder, ops};
use std::path::PathBuf;

/// Configurazione Qwen3.5 (da config.json)
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Qwen35Config {
    pub hidden_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub intermediate_size: usize,
    pub vocab_size: usize,
    pub max_position_embeddings: usize,
    pub head_dim: usize,
    pub rms_norm_eps: f64,
    pub rope_theta: f64,
    pub partial_rotary_factor: f32,
    
    // Qwen3.5 specific
    pub layer_types: Vec<String>,  // "linear_attention" o "full_attention"
    pub full_attention_interval: usize,
    pub linear_num_key_heads: usize,
    pub linear_num_value_heads: usize,
    pub linear_key_head_dim: usize,
    pub linear_value_head_dim: usize,
    pub linear_conv_kernel_dim: usize,
    pub attn_output_gate: bool,
}

impl Qwen35Config {
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let mut config: serde_json::Value = serde_json::from_str(&json)?;
        
        // Qwen3.5 ha text_config nested
        if let Some(text_config) = config.get_mut("text_config") {
            serde_json::from_value(text_config.clone())
                .context("Failed to parse text_config")
        } else {
            serde_json::from_value(config)
                .context("Failed to parse config")
        }
    }
}

/// RMSNorm (Root Mean Square Layer Normalization)
struct RmsNorm {
    weight: Tensor,
    eps: f64,
}

impl RmsNorm {
    fn new(weight: Tensor, eps: f64) -> Self {
        Self { weight, eps }
    }
    
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x_dtype = x.dtype();
        let internal_dtype = match x_dtype {
            DType::F16 | DType::BF16 => DType::F32,
            d => d,
        };
        
        let hidden_size = x.dim(x.dims().len() - 1)?;
        let x = x.to_dtype(internal_dtype)?;
        
        // RMS = sqrt(mean(x^2) + eps)
        let norm_x = (x.sqr()?.sum_keepdim(x.dims().len() - 1)? / hidden_size as f64)?;
        let x_normed = x.broadcast_div(&(norm_x + self.eps)?.sqrt()?)?;
        
        let x_normed = x_normed.to_dtype(x_dtype)?;
        Ok(x_normed.broadcast_mul(&self.weight)?)
    }
}

/// Rotary Position Embeddings (RoPE) con mRoPE per Qwen3.5
struct RotaryEmbedding {
    sin: Tensor,
    cos: Tensor,
    dim: usize,
}

impl RotaryEmbedding {
    fn new(cfg: &Qwen35Config, device: &Device) -> Result<Self> {
        let dim = (cfg.head_dim as f32 * cfg.partial_rotary_factor) as usize;
        let max_seq_len = cfg.max_position_embeddings;
        let theta = cfg.rope_theta as f32;
        
        // Calcola frequenze: theta^(-2i/dim) per i in [0, dim/2)
        let inv_freq: Vec<f32> = (0..dim)
            .step_by(2)
            .map(|i| 1.0 / theta.powf(i as f32 / dim as f32))
            .collect();
        
        let inv_freq_len = inv_freq.len();
        let inv_freq = Tensor::from_vec(inv_freq, (1, inv_freq_len), device)?;
        
        // Posizioni: [0, 1, 2, ..., max_seq_len-1]
        let t: Vec<f32> = (0..max_seq_len).map(|i| i as f32).collect();
        let t = Tensor::from_vec(t, (max_seq_len, 1), device)?;
        
        // freqs = t @ inv_freq -> [max_seq_len, dim/2]
        let freqs = t.matmul(&inv_freq)?;
        
        // Duplica per avere [max_seq_len, dim]
        let freqs = Tensor::cat(&[&freqs, &freqs], 1)?;
        
        let cos = freqs.cos()?;
        let sin = freqs.sin()?;
        
        Ok(Self { sin, cos, dim })
    }
    
    fn apply_rotary(&self, q: &Tensor, k: &Tensor, seq_len: usize) -> Result<(Tensor, Tensor)> {
        let cos = self.cos.narrow(0, 0, seq_len)?;
        let sin = self.sin.narrow(0, 0, seq_len)?;
        
        let q_rot = Self::rotate_half(q, &cos, &sin)?;
        let k_rot = Self::rotate_half(k, &cos, &sin)?;
        
        Ok((q_rot, k_rot))
    }
    
    fn rotate_half(x: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
        let last_dim = x.dims().len() - 1;
        let dim = x.dim(last_dim)?;
        let half_dim = dim / 2;
        
        let x1 = x.narrow(last_dim, 0, half_dim)?;
        let x2 = x.narrow(last_dim, half_dim, half_dim)?;
        
        // rotate: [x1*cos - x2*sin, x1*sin + x2*cos]
        let rotated1 = (x1.broadcast_mul(cos)? - x2.broadcast_mul(sin)?)?;
        let rotated2 = (x1.broadcast_mul(sin)? + x2.broadcast_mul(cos)?)?;
        
        Ok(Tensor::cat(&[rotated1, rotated2], last_dim)?)
    }
}

/// MLP con SwiGLU activation
struct Mlp {
    gate_proj: Tensor,
    up_proj: Tensor,
    down_proj: Tensor,
}

impl Mlp {
    fn new(vb: VarBuilder, cfg: &Qwen35Config, layer_idx: usize) -> Result<Self> {
        let hidden = cfg.hidden_size;
        let intermediate = cfg.intermediate_size;
        
        let gate_proj = vb.pp(&format!("model.layers.{}.mlp.gate_proj", layer_idx))
            .get((intermediate, hidden), "weight")?;
        let up_proj = vb.pp(&format!("model.layers.{}.mlp.up_proj", layer_idx))
            .get((intermediate, hidden), "weight")?;
        let down_proj = vb.pp(&format!("model.layers.{}.mlp.down_proj", layer_idx))
            .get((hidden, intermediate), "weight")?;
        
        Ok(Self { gate_proj, up_proj, down_proj })
    }
    
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        // SwiGLU: down(silu(gate(x)) * up(x))
        let gate = x.matmul(&self.gate_proj.t()?)?;
        let gate_act = ops::silu(&gate)?;
        let up = x.matmul(&self.up_proj.t()?)?;
        let mlp_out = (gate_act * up)?.matmul(&self.down_proj.t()?)?;
        Ok(mlp_out)
    }
}

/// Full Attention (standard multi-head attention)
struct FullAttention {
    q_proj: Tensor,
    k_proj: Tensor,
    v_proj: Tensor,
    o_proj: Tensor,
    num_heads: usize,
    num_kv_heads: usize,
    head_dim: usize,
    rope: RotaryEmbedding,
}

impl FullAttention {
    fn new(vb: VarBuilder, cfg: &Qwen35Config, layer_idx: usize, rope: RotaryEmbedding) -> Result<Self> {
        let hidden = cfg.hidden_size;
        let num_heads = cfg.num_attention_heads;
        let num_kv_heads = cfg.num_key_value_heads;
        let head_dim = cfg.head_dim;
        
        let q_proj = vb.pp(&format!("model.layers.{}.self_attn.q_proj", layer_idx))
            .get((num_heads * head_dim, hidden), "weight")?;
        let k_proj = vb.pp(&format!("model.layers.{}.self_attn.k_proj", layer_idx))
            .get((num_kv_heads * head_dim, hidden), "weight")?;
        let v_proj = vb.pp(&format!("model.layers.{}.self_attn.v_proj", layer_idx))
            .get((num_kv_heads * head_dim, hidden), "weight")?;
        let o_proj = vb.pp(&format!("model.layers.{}.self_attn.o_proj", layer_idx))
            .get((hidden, num_heads * head_dim), "weight")?;
        
        Ok(Self {
            q_proj,
            k_proj,
            v_proj,
            o_proj,
            num_heads,
            num_kv_heads,
            head_dim,
            rope,
        })
    }
    
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let (batch, seq_len, hidden) = x.dims3()?;
        
        // QKV projections
        let q = x.matmul(&self.q_proj.t()?)?;
        let k = x.matmul(&self.k_proj.t()?)?;
        let v = x.matmul(&self.v_proj.t()?)?;
        
        // Reshape per multi-head: [batch, seq, num_heads, head_dim]
        let q = q.reshape((batch, seq_len, self.num_heads, self.head_dim))?
            .transpose(1, 2)?; // [batch, num_heads, seq, head_dim]
        let k = k.reshape((batch, seq_len, self.num_kv_heads, self.head_dim))?
            .transpose(1, 2)?;
        let v = v.reshape((batch, seq_len, self.num_kv_heads, self.head_dim))?
            .transpose(1, 2)?;
        
        // Apply RoPE
        let (q, k) = self.rope.apply_rotary(&q, &k, seq_len)?;
        
        // Grouped-query attention (se num_kv_heads < num_heads)
        let k = self.repeat_kv(k, self.num_heads / self.num_kv_heads)?;
        let v = self.repeat_kv(v, self.num_heads / self.num_kv_heads)?;
        
        // Attention scores: Q @ K^T / sqrt(head_dim)
        let scale = (self.head_dim as f64).sqrt();
        let attn_scores = q.matmul(&k.transpose(2, 3)?)?;
        let attn_scores = (attn_scores / scale)?;
        
        // Softmax
        let attn_weights = ops::softmax_last_dim(&attn_scores)?;
        
        // Attention output: weights @ V
        let attn_output = attn_weights.matmul(&v)?; // [batch, num_heads, seq, head_dim]
        
        // Reshape back: [batch, seq, hidden]
        let attn_output = attn_output.transpose(1, 2)?
            .reshape((batch, seq_len, self.num_heads * self.head_dim))?;
        
        // Output projection
        let output = attn_output.matmul(&self.o_proj.t()?)?;
        
        Ok(output)
    }
    
    fn repeat_kv(&self, x: Tensor, n_rep: usize) -> Result<Tensor> {
        if n_rep == 1 {
            return Ok(x);
        }
        
        let (batch, num_kv_heads, seq_len, head_dim) = x.dims4()?;
        
        // Repeat: [batch, num_kv_heads, seq, head_dim] -> [batch, num_kv_heads*n_rep, seq, head_dim]
        let x = x.unsqueeze(2)?; // [batch, num_kv_heads, 1, seq, head_dim]
        let x = x.expand(&[batch, num_kv_heads, n_rep, seq_len, head_dim])?;
        let x = x.reshape((batch, num_kv_heads * n_rep, seq_len, head_dim))?;
        
        Ok(x)
    }
}

/// Linear Attention (efficiente per sequenze lunghe)
/// Implementazione semplificata - la versione completa richiede conv1d e gating
struct LinearAttention {
    in_proj_qkv: Tensor,
    out_proj: Tensor,
    num_key_heads: usize,
    num_value_heads: usize,
    key_head_dim: usize,
    value_head_dim: usize,
}

impl LinearAttention {
    fn new(vb: VarBuilder, cfg: &Qwen35Config, layer_idx: usize) -> Result<Self> {
        let hidden = cfg.hidden_size;
        let num_key_heads = cfg.linear_num_key_heads;
        let num_value_heads = cfg.linear_num_value_heads;
        let key_head_dim = cfg.linear_key_head_dim;
        let value_head_dim = cfg.linear_value_head_dim;
        
        let qkv_dim = num_key_heads * key_head_dim * 2 + num_value_heads * value_head_dim;
        
        let in_proj_qkv = vb.pp(&format!("model.layers.{}.linear_attn.in_proj_qkv", layer_idx))
            .get((qkv_dim, hidden), "weight")?;
        let out_proj = vb.pp(&format!("model.layers.{}.linear_attn.out_proj", layer_idx))
            .get((hidden, num_value_heads * value_head_dim), "weight")?;
        
        Ok(Self {
            in_proj_qkv,
            out_proj,
            num_key_heads,
            num_value_heads,
            key_head_dim,
            value_head_dim,
        })
    }
    
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        // Linear attention: O(n) invece di O(n²)
        // Implementazione semplificata: usa solo proiezione lineare
        // La versione completa richiede kernel attention e conv1d
        
        let (batch, seq_len, _hidden) = x.dims3()?;
        
        // QKV projection
        let qkv = x.matmul(&self.in_proj_qkv.t()?)?;
        
        // Split in Q, K, V
        let qk_dim = self.num_key_heads * self.key_head_dim;
        let v_dim = self.num_value_heads * self.value_head_dim;
        
        let q = qkv.narrow(2, 0, qk_dim)?;
        let k = qkv.narrow(2, qk_dim, qk_dim)?;
        let v = qkv.narrow(2, qk_dim * 2, v_dim)?;
        
        // Linear attention kernel (semplificato)
        // Versione completa: kernel_fn(Q) @ kernel_fn(K)^T @ V
        // Qui: semplice weighted sum
        
        let k_sum = k.sum_keepdim(1)?; // [batch, 1, qk_dim]
        let weights = ops::softmax(&k_sum, 2)?;
        let context = v.broadcast_mul(&weights.broadcast_as(v.shape())?)?;
        let context = context.sum(1)?; // [batch, v_dim]
        let context = context.unsqueeze(1)?
            .expand(&[batch, seq_len, v_dim])?;
        
        // Output projection
        let output = context.matmul(&self.out_proj.t()?)?;
        
        Ok(output)
    }
}

/// Singolo layer Qwen3.5 (può essere Linear o Full Attention)
struct Qwen35Layer {
    input_layernorm: RmsNorm,
    post_attention_layernorm: RmsNorm,
    attention: AttentionType,
    mlp: Mlp,
}

enum AttentionType {
    Linear(LinearAttention),
    Full(FullAttention),
}

impl Qwen35Layer {
    fn new(vb: VarBuilder, cfg: &Qwen35Config, layer_idx: usize, rope: RotaryEmbedding) -> Result<Self> {
        let eps = cfg.rms_norm_eps;
        
        // Layer norms
        let input_ln_weight = vb.pp(&format!("model.layers.{}.input_layernorm", layer_idx))
            .get(cfg.hidden_size, "weight")?;
        let post_ln_weight = vb.pp(&format!("model.layers.{}.post_attention_layernorm", layer_idx))
            .get(cfg.hidden_size, "weight")?;
        
        let input_layernorm = RmsNorm::new(input_ln_weight, eps);
        let post_attention_layernorm = RmsNorm::new(post_ln_weight, eps);
        
        // Attention (linear o full)
        let layer_type = &cfg.layer_types[layer_idx];
        let attention = if layer_type == "full_attention" {
            AttentionType::Full(FullAttention::new(vb.clone(), cfg, layer_idx, rope)?)
        } else {
            AttentionType::Linear(LinearAttention::new(vb.clone(), cfg, layer_idx)?)
        };
        
        // MLP
        let mlp = Mlp::new(vb, cfg, layer_idx)?;
        
        Ok(Self {
            input_layernorm,
            post_attention_layernorm,
            attention,
            mlp,
        })
    }
    
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        // Pre-norm architecture
        let residual = x.clone();
        
        // Attention block
        let normed = self.input_layernorm.forward(x)?;
        let attn_out = match &self.attention {
            AttentionType::Linear(attn) => attn.forward(&normed)?,
            AttentionType::Full(attn) => attn.forward(&normed)?,
        };
        let hidden = (residual + attn_out)?;
        
        // MLP block
        let residual = hidden.clone();
        let normed = self.post_attention_layernorm.forward(&hidden)?;
        let mlp_out = self.mlp.forward(&normed)?;
        let output = (residual + mlp_out)?;
        
        Ok(output)
    }
}

/// Modello Qwen3.5 completo
pub struct Qwen35Model {
    embed_tokens: Tensor,
    layers: Vec<Qwen35Layer>,
    norm: RmsNorm,
    config: Qwen35Config,
    device: Device,
}

impl Qwen35Model {
    pub fn load(model_files: &[PathBuf], config: Qwen35Config, device: Device) -> Result<Self> {
        println!("[Qwen3.5] Caricamento pesi da {} file...", model_files.len());
        
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(model_files, DType::F32, &device)?
        };
        
        // Embedding
        let embed_tokens = vb.pp("model.embed_tokens")
            .get((config.vocab_size, config.hidden_size), "weight")?;
        
        println!("[Qwen3.5] Embeddings caricati: {} × {}", config.vocab_size, config.hidden_size);
        
        // Layers
        let rope = RotaryEmbedding::new(&config, &device)?;
        let mut layers = Vec::with_capacity(config.num_hidden_layers);
        
        for i in 0..config.num_hidden_layers {
            let layer = Qwen35Layer::new(vb.clone(), &config, i, rope.clone())?;
            layers.push(layer);
            
            if (i + 1) % 8 == 0 {
                println!("[Qwen3.5] Layer {}/{} caricati", i + 1, config.num_hidden_layers);
            }
        }
        
        // Final norm
        let norm_weight = vb.pp("model.norm").get(config.hidden_size, "weight")?;
        let norm = RmsNorm::new(norm_weight, config.rms_norm_eps);
        
        println!("[Qwen3.5] Modello completo caricato");
        
        Ok(Self {
            embed_tokens,
            layers,
            norm,
            config,
            device,
        })
    }
    
    /// Forward pass con estrazione hidden states
    pub fn forward(&self, input_ids: &Tensor, extract_layer: Option<usize>) -> Result<Tensor> {
        let (batch, seq_len) = input_ids.dims2()?;
        
        // Embedding
        let mut hidden = self.embed_tokens.index_select(input_ids, 0)?
            .reshape((batch, seq_len, self.config.hidden_size))?;
        
        // Layer-by-layer forward
        let target_layer = extract_layer.unwrap_or(self.config.num_hidden_layers - 1);
        
        for (i, layer) in self.layers.iter().enumerate() {
            hidden = layer.forward(&hidden)?;
            
            if i == target_layer {
                break;
            }
        }
        
        // Final norm
        let hidden = self.norm.forward(&hidden)?;
        
        Ok(hidden)
    }
}

// Helper per clonare RotaryEmbedding
impl Clone for RotaryEmbedding {
    fn clone(&self) -> Self {
        Self {
            sin: self.sin.clone(),
            cos: self.cos.clone(),
            dim: self.dim,
        }
    }
}
