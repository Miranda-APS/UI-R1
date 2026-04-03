// src/topology/llm_substrate.rs
//
// LLM Substrate — L'LLM abita lo spazio topologico di Prometeo
//
// Filosofia:
// - L'LLM non è un'interfaccia esterna, è il substrato fisico dove il campo esiste
// - Gli hidden states del modello SONO il campo topologico
// - Le 8 dimensioni di Prometeo sono proiezioni degli hidden states
// - La memoria è SOLO quella di Prometeo (episodic.rs, memory.rs)
// - NO cache del modello, ogni forward pass parte dallo stato di Prometeo

use anyhow::{Result, Context as AnyhowContext};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::topology::{
    engine::PrometeoTopologyEngine,
    primitive::PrimitiveCore,
    fractal::FractalId,
    identity::IdentityCore,
};

/// Risposta generata dall'LLM che abita il campo topologico
#[derive(Debug, Clone)]
pub struct InhabitedResponse {
    /// Firma 8D estratta dagli hidden states
    pub field_signature: [f32; 8],
    
    /// Testo generato (vincolato al lessico di Prometeo)
    pub text: String,
    
    /// Attivazioni per layer (opzionale, per debug)
    pub layer_activations: Option<Vec<Vec<f32>>>,
    
    /// Frattale dominante emergente
    pub dominant_fractal: Option<FractalId>,
    
    /// Parole generate che erano nel lessico
    pub known_words: Vec<String>,
    
    /// Parole generate sconosciute (da apprendere)
    pub unknown_words: Vec<String>,
}

/// Configurazione del substrato LLM
#[derive(Debug, Clone)]
pub struct SubstrateConfig {
    /// Modello da usare (es. "Qwen/Qwen3.5-9B")
    pub model_name: String,
    
    /// Device (CPU, CUDA, Metal)
    pub device: DeviceType,
    
    /// Layer da cui estrarre gli hidden states (default: ultimo)
    pub extraction_layer: Option<usize>,
    
    /// Temperatura per sampling
    pub temperature: f32,
    
    /// Top-p per nucleus sampling
    pub top_p: f32,
    
    /// Max token da generare
    pub max_tokens: usize,
    
    /// Vincola generazione solo a parole del lessico
    pub constrain_to_lexicon: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceType {
    CPU,
    CUDA(usize),  // GPU id
    Metal,
}

impl Default for SubstrateConfig {
    fn default() -> Self {
        Self {
            model_name: "Qwen/Qwen3.5-9B".to_string(),
            device: DeviceType::CPU,
            extraction_layer: None,  // usa ultimo layer
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 100,
            constrain_to_lexicon: true,
        }
    }
}

/// Il substrato LLM — dove Prometeo abita
pub struct LLMSubstrate {
    config: SubstrateConfig,
    
    // Matrice di proiezione: [hidden_dim, 8]
    // Mappa gli hidden states sulle 8 dimensioni di Prometeo
    projection_matrix: ndarray::Array2<f32>,
    
    // Vocabolario: token_id → parola
    // Solo token che corrispondono a parole nel lessico di Prometeo
    allowed_tokens: HashSet<u32>,
    token_to_word: HashMap<u32, String>,
    
    // Stato di calibrazione
    is_calibrated: bool,
    calibration_samples: usize,
}

impl LLMSubstrate {
    /// Crea un nuovo substrato (non ancora calibrato)
    pub fn new(config: SubstrateConfig) -> Self {
        // Dimensione hidden dipende dal modello
        // Qwen3.5-4B: 2048, Qwen3.5-9B: 4096, Qwen3.5-27B: 4608
        let hidden_dim = 4096;  // default per Qwen3.5-9B, verrà aggiornato al primo forward
        
        // Inizializza proiezione casuale (verrà calibrata)
        let projection = ndarray::Array2::from_shape_fn((hidden_dim, 8), |(_, _)| {
            rand::random::<f32>() * 0.1 - 0.05  // piccoli valori casuali
        });
        
        Self {
            config,
            projection_matrix: projection,
            allowed_tokens: HashSet::new(),
            token_to_word: HashMap::new(),
            is_calibrated: false,
            calibration_samples: 0,
        }
    }
    
    /// Carica la matrice di proiezione da file .npy (generato da calibrate_llm_projection.py)
    pub fn load_projection(&mut self, npy_path: &std::path::Path) -> Result<()> {
        use std::fs::File;
        use std::io::Read;
        
        println!("[LLM Substrate] Caricamento matrice da {:?}...", npy_path);
        
        let mut file = File::open(npy_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        
        // Parse .npy header (formato numpy)
        let (shape, data_offset) = Self::parse_npy_header(&buffer)?;
        
        if shape.len() != 2 {
            anyhow::bail!("Expected 2D array, got {}D", shape.len());
        }
        
        let (rows, cols) = (shape[0], shape[1]);
        if cols != 8 {
            anyhow::bail!("Expected 8 output dimensions, got {}", cols);
        }
        
        println!("  Shape: {} × {}", rows, cols);
        
        // Leggi i dati float32
        let data_bytes = &buffer[data_offset..];
        let float_count = rows * cols;
        
        if data_bytes.len() < float_count * 4 {
            anyhow::bail!("Insufficient data in .npy file");
        }
        
        let mut data = Vec::with_capacity(float_count);
        for i in 0..float_count {
            let offset = i * 4;
            let bytes = [
                data_bytes[offset],
                data_bytes[offset + 1],
                data_bytes[offset + 2],
                data_bytes[offset + 3],
            ];
            data.push(f32::from_le_bytes(bytes));
        }
        
        // Crea matrice [rows, 8]
        self.projection_matrix = ndarray::Array2::from_shape_vec((rows, cols), data)?;
        self.is_calibrated = true;
        
        println!("✓ Matrice caricata: {} × {}", rows, cols);
        
        Ok(())
    }
    
    /// Parse header .npy (formato numpy)
    fn parse_npy_header(buffer: &[u8]) -> Result<(Vec<usize>, usize)> {
        // Magic: \x93NUMPY
        if buffer.len() < 10 || &buffer[0..6] != b"\x93NUMPY" {
            anyhow::bail!("Invalid .npy magic");
        }
        
        let major = buffer[6];
        let minor = buffer[7];
        
        if major != 1 && major != 2 {
            anyhow::bail!("Unsupported .npy version {}.{}", major, minor);
        }
        
        // Header length
        let header_len = if major == 1 {
            u16::from_le_bytes([buffer[8], buffer[9]]) as usize
        } else {
            u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]) as usize
        };
        
        let header_start = if major == 1 { 10 } else { 12 };
        let header_end = header_start + header_len;
        
        if buffer.len() < header_end {
            anyhow::bail!("Truncated .npy header");
        }
        
        let header_str = std::str::from_utf8(&buffer[header_start..header_end])?;
        
        // Parse shape da header (formato: "{'descr': '<f4', 'fortran_order': False, 'shape': (4096, 8), }")
        let shape = Self::extract_shape_from_header(header_str)?;
        
        Ok((shape, header_end))
    }
    
    fn extract_shape_from_header(header: &str) -> Result<Vec<usize>> {
        // Cerca 'shape': (dim1, dim2, ...)
        let shape_start = header.find("'shape':")
            .ok_or_else(|| anyhow::anyhow!("No 'shape' in header"))?;
        
        let tuple_start = header[shape_start..].find('(')
            .ok_or_else(|| anyhow::anyhow!("No '(' after 'shape'"))?;
        
        let tuple_end = header[shape_start + tuple_start..].find(')')
            .ok_or_else(|| anyhow::anyhow!("No ')' in shape tuple"))?;
        
        let tuple_content = &header[shape_start + tuple_start + 1..shape_start + tuple_start + tuple_end];
        
        let dims: Result<Vec<usize>> = tuple_content
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<usize>().map_err(|e| anyhow::anyhow!("Parse error: {}", e)))
            .collect();
        
        dims
    }
    
    /// Calibra la matrice di proiezione usando il lessico di Prometeo
    ///
    /// DEPRECATO: usa invece lo script Python tools/calibrate_llm_projection.py
    /// e poi carica con load_projection()
    pub fn calibrate(&mut self, engine: &PrometeoTopologyEngine) -> Result<CalibrationReport> {
        println!("[LLM Substrate] DEPRECATO: usa tools/calibrate_llm_projection.py");
        
        let stable_words: Vec<_> = engine.lexicon.patterns_iter()
            .filter(|(_, w)| w.stability > 0.5 && w.exposure_count >= 10)
            .take(1000)
            .collect();
        
        Ok(CalibrationReport {
            words_used: stable_words.len(),
            mean_error: 0.0,
            calibration_quality: 0.0,
        })
    }
    
    /// L'LLM abita il campo di Prometeo e genera una risposta
    ///
    /// Flusso:
    /// 1. Serializza lo stato di Prometeo (campo, memoria, identità)
    /// 2. Forward pass nel modello (NO cache, parte da zero)
    /// 3. Estrai hidden states dal layer specificato
    /// 4. Proietta sulle 8 dimensioni
    /// 5. Genera token vincolati al lessico
    /// 6. Ritorna risposta + firma 8D
    pub fn inhabit(&mut self, engine: &PrometeoTopologyEngine) -> Result<InhabitedResponse> {
        if !self.is_calibrated {
            anyhow::bail!("Substrato non calibrato. Chiama calibrate() prima di inhabit()");
        }
        
        // 1. Costruisci il contesto dalla memoria di Prometeo
        let context = Self::build_context_from_prometeo(engine);
        
        println!("[LLM Substrate] Context length: {} chars", context.len());
        
        // 2. TODO: Forward pass con Candle
        // let hidden_states = self.model.forward_with_hidden_states(&tokens)?;
        
        // 3. TODO: Proietta sulle 8 dimensioni
        // let field_8d = self.project_to_8d(&hidden_states)?;
        
        // 4. TODO: Genera vincolato al lessico
        // let generated = self.generate_constrained(&logits, engine)?;
        
        // Placeholder per ora
        let field_8d = [0.5; 8];
        let text = "risposta placeholder".to_string();
        
        Ok(InhabitedResponse {
            field_signature: field_8d,
            text,
            layer_activations: None,
            dominant_fractal: None,
            known_words: vec![],
            unknown_words: vec![],
        })
    }
    
    /// Costruisce il prompt dal
    /// Costruisce il prompt dallo stato di Prometeo
    ///
    /// CRITICO: usa SOLO la memoria di Prometeo, non la cache del modello
    fn build_context_from_prometeo(engine: &PrometeoTopologyEngine) -> String {
        let mut context = String::new();
        
        // Stato del campo (firma 8D)
        let field_sig = engine.field_sig();
        context.push_str(&format!("CAMPO: {:?}\n", field_sig));
        
        // Frattali attivi
        let active_fractals = engine.active_fractals();
        if !active_fractals.is_empty() {
            context.push_str("FRATTALI ATTIVI:\n");
            for (name, activation) in active_fractals.iter().take(5) {
                context.push_str(&format!("  {} ({:.2})\n", name, activation));
            }
        }
        
        // Memoria episodica (ultimi 5 turni)
        // Questo è l'UNICO contesto conversazionale — dalla memoria di Prometeo
        context.push_str("\nMEMORIA:\n");
        // TODO: accesso a episodic_memory
        // for trace in engine.episodic_memory.recent_traces(5) {
        //     context.push_str(&format!("  [{}] {}\n", trace.speaker, trace.phrase));
        // }
        
        // Identità olografica
        if let Some(projection) = engine.holographic_projection() {
            context.push_str(&format!("\nIDENTITÀ: {:?}\n", projection));
        }
        
        // Volontà corrente
        if let Some(will) = engine.current_will() {
            context.push_str(&format!("VOLONTÀ: {:?}\n", will.intention));
        }
        
        // Parole più attive nel campo (top 20)
        context.push_str("\nPAROLE ATTIVE:\n");
        let active_words = engine.what_i_see();
        for (word, activation) in active_words.iter().take(20) {
            context.push_str(&format!("  {} ({:.3})\n", word, activation));
        }
        
        context.push_str("\n---\n");
        context
    }
    
    /// Proietta hidden states [hidden_dim] sulle 8 dimensioni di Prometeo
    fn project_to_8d(&self, hidden: &[f32]) -> Result<[f32; 8]> {
        if hidden.len() != self.projection_matrix.nrows() {
            anyhow::bail!(
                "Hidden dim mismatch: expected {}, got {}",
                self.projection_matrix.nrows(),
                hidden.len()
            );
        }
        
        let hidden_array = ndarray::Array1::from_vec(hidden.to_vec());
        let projected = self.projection_matrix.t().dot(&hidden_array);
        
        // Normalizza in [0, 1]
        let mut result = [0.0f32; 8];
        for i in 0..8 {
            result[i] = (projected[i].tanh() + 1.0) / 2.0;  // tanh → [0, 1]
        }
        
        Ok(result)
    }
    
    /// Genera token vincolati al lessico di Prometeo
    ///
    /// Maschera tutti i token che non corrispondono a parole conosciute
    fn generate_constrained(&self, 
                           _logits: &[f32], 
                           _engine: &PrometeoTopologyEngine) -> Result<Vec<u32>> {
        // TODO: implementazione completa
        // 1. Per ogni token_id in logits
        // 2. Se token_id non in allowed_tokens: logit = -inf
        // 3. Sample da distribuzione mascherata
        
        Ok(vec![])
    }
    
    /// Costruisce il vocabolario consentito dal lessico di Prometeo
    /// NOTA: Questa versione è per LLMSubstrate (senza tokenizer)
    /// La versione completa è in CandleSubstrate
    pub fn build_allowed_vocabulary(&mut self, _engine: &PrometeoTopologyEngine) -> Result<usize> {
        // Placeholder - la versione reale è in CandleSubstrate
        Ok(0)
    }
}

#[derive(Debug)]
pub struct CalibrationReport {
    pub words_used: usize,
    pub mean_error: f64,
    pub calibration_quality: f64,
}

// ============================================================================
// IMPLEMENTAZIONE CANDLE
// ============================================================================

#[cfg(feature = "llm-substrate")]
pub mod candle_impl {
    use super::*;
    use candle_core::{Device, Tensor, DType};
    use candle_nn::VarBuilder;
    use hf_hub::{api::tokio::Api, Repo, RepoType};
    use tokenizers::Tokenizer;
    use std::path::PathBuf;
    
    use crate::topology::llm_substrate_qwen35::{Qwen35Model, Qwen35Config};
    
    pub struct CandleSubstrate {
        model: Qwen35Model,
        tokenizer: Tokenizer,
        device: Device,
        config: super::SubstrateConfig,
        projection: ndarray::Array2<f32>,
        allowed_tokens: HashSet<u32>,
        is_calibrated: bool,
    }
    
    impl CandleSubstrate {
        /// Carica il modello da HuggingFace in formato SafeTensors
        pub async fn load(config: super::SubstrateConfig) -> Result<Self> {
            println!("[Candle] Caricamento modello: {}", config.model_name);
            
            // 1. Setup device
            let device = match config.device {
                super::DeviceType::CPU => Device::Cpu,
                super::DeviceType::CUDA(id) => Device::new_cuda(id)?,
                super::DeviceType::Metal => Device::new_metal(0)?,
            };
            
            // 2. Download da HuggingFace
            let api = Api::new()?;
            let repo = api.repo(Repo::new(
                config.model_name.clone(),
                RepoType::Model,
            ));
            
            println!("[Candle] Download file da HuggingFace...");
            
            // Scarica i file necessari
            let config_file = repo.get("config.json").await?;
            let tokenizer_file = repo.get("tokenizer.json").await?;
            
            // SafeTensors files (possono essere multipli)
            let model_files = Self::get_safetensors_files(&repo).await?;
            
            println!("[Candle] File scaricati: {} SafeTensors", model_files.len());
            
            // 3. Carica config
            let model_config = Qwen35Config::from_file(&config_file)?;
            
            println!("[Candle] Config: hidden_size={}, layers={}", 
                     model_config.hidden_size, model_config.num_hidden_layers);
            
            // 4. Carica tokenizer
            let tokenizer = Tokenizer::from_file(tokenizer_file)
                .map_err(|e| anyhow::anyhow!("Tokenizer error: {}", e))?;
            
            println!("[Candle] Tokenizer caricato, vocab_size={}", tokenizer.get_vocab_size(true));
            
            // 5. Carica modello
            let model = Qwen35Model::load(&model_files, model_config.clone(), device.clone())?;
            
            // 6. Inizializza proiezione (verrà calibrata)
            let hidden_dim = model_config.hidden_size;
            let projection = ndarray::Array2::from_shape_fn((hidden_dim, 8), |(_, _)| {
                (rand::random::<f32>() - 0.5) * 0.1
            });
            
            println!("[Candle] Substrato inizializzato");
            
            Ok(Self {
                model,
                tokenizer,
                device,
                config,
                projection,
                allowed_tokens: HashSet::new(),
                is_calibrated: false,
            })
        }
        
        async fn get_safetensors_files(repo: &Repo) -> Result<Vec<PathBuf>> {
            // Cerca tutti i file model.safetensors*
            let mut files = Vec::new();
            
            // Prova prima file singolo
            if let Ok(file) = repo.get("model.safetensors").await {
                files.push(file);
                return Ok(files);
            }
            
            // Altrimenti cerca sharded files
            let mut idx = 1;
            loop {
                let filename = format!("model.safetensors-{:05}-of-{:05}.safetensors", idx, 4);
                match repo.get(&filename).await {
                    Ok(file) => {
                        files.push(file);
                        idx += 1;
                        if idx > 10 {
                            break; // safety limit
                        }
                    }
                    Err(_) => break,
                }
            }
            
            if files.is_empty() {
                anyhow::bail!("Nessun file SafeTensors trovato");
            }
            
            Ok(files)
        }
        
        /// Carica la matrice di proiezione calibrata
        pub fn load_projection(&mut self, path: &std::path::Path) -> Result<()> {
            println!("[Candle] Caricamento proiezione da {:?}", path);
            
            // Leggi il file .npy usando ndarray-npy
            // Il file contiene una matrice [hidden_dim, 8] in formato numpy
            use std::fs::File;
            use std::io::BufReader;
            
            let file = File::open(path)
                .with_context(|| format!("Impossibile aprire {}", path.display()))?;
            let _reader = BufReader::new(file);
            
            // Usa ndarray-npy per leggere direttamente
            // NOTA: richiede dipendenza ndarray-npy in Cargo.toml
            // Per ora, implementiamo parsing manuale del formato .npy
            
            // Formato .npy header:
            // - Magic: b"\x93NUMPY"
            // - Version: 1 byte major, 1 byte minor
            // - Header len: 2 byte (little endian)
            // - Header dict: Python dict con dtype, shape, fortran_order
            // - Data: raw bytes
            
            // Implementazione semplificata: assumiamo float32, C-order
            let bytes = std::fs::read(path)?;
            
            // Verifica magic
            if bytes.len() < 10 || &bytes[0..6] != b"\x93NUMPY" {
                anyhow::bail!("File non è un .npy valido");
            }
            
            // Leggi header length (byte 8-9, little endian)
            let header_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
            let header_end = 10 + header_len;
            
            if bytes.len() < header_end {
                anyhow::bail!("File .npy troncato");
            }
            
            // Parse header (formato Python dict)
            let header_str = std::str::from_utf8(&bytes[10..header_end])?;
            
            // Estrai shape: cerca 'shape': (dim1, dim2)
            let shape_start = header_str.find("'shape':")
                .or_else(|| header_str.find("\"shape\":"))
                .ok_or_else(|| anyhow::anyhow!("shape non trovato in header"))?;
            let shape_str = &header_str[shape_start..];
            let shape_tuple_start = shape_str.find('(')
                .ok_or_else(|| anyhow::anyhow!("shape tuple non trovato"))?;
            let shape_tuple_end = shape_str.find(')')
                .ok_or_else(|| anyhow::anyhow!("shape tuple non chiuso"))?;
            let shape_content = &shape_str[shape_tuple_start+1..shape_tuple_end];
            
            let dims: Vec<usize> = shape_content.split(',')
                .filter_map(|s| s.trim().parse::<usize>().ok())
                .collect();
            
            if dims.len() != 2 {
                anyhow::bail!("Shape deve essere 2D, trovato: {:?}", dims);
            }
            
            let (rows, cols) = (dims[0], dims[1]);
            
            if cols != 8 {
                anyhow::bail!("Proiezione deve avere 8 colonne, trovato: {}", cols);
            }
            
            // Leggi i dati (float32, little endian)
            let data_start = header_end;
            let expected_bytes = rows * cols * 4; // 4 byte per float32
            
            if bytes.len() < data_start + expected_bytes {
                anyhow::bail!("Dati insufficienti nel file .npy");
            }
            
            let mut matrix_data = Vec::with_capacity(rows * cols);
            for i in 0..(rows * cols) {
                let offset = data_start + i * 4;
                let float_bytes = [
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ];
                matrix_data.push(f32::from_le_bytes(float_bytes));
            }
            
            // Crea la matrice ndarray
            self.projection = ndarray::Array2::from_shape_vec((rows, cols), matrix_data)?;
            
            println!("[Candle] Proiezione caricata: {} × {}", rows, cols);
            
            Ok(())
        }
        
        /// Forward pass con estrazione hidden states
        pub fn forward_with_states(&self, input_text: &str) -> Result<ForwardOutput> {
            // 1. Tokenizza
            let encoding = self.tokenizer
                .encode(input_text, false)
                .map_err(|e| anyhow::anyhow!("Encoding error: {}", e))?;
            
            let tokens = encoding.get_ids();
            
            if tokens.is_empty() {
                anyhow::bail!("Input vuoto dopo tokenizzazione");
            }
            
            println!("[Candle] Tokenizzato: {} token", tokens.len());
            
            // 2. Converti token IDs in tensor
            let token_ids: Vec<u32> = tokens.to_vec();
            let input_ids = Tensor::new(token_ids.as_slice(), &self.device)?
                .unsqueeze(0)?; // [1, seq_len]
            
            // 3. Forward pass attraverso il modello
            let target_layer = self.config.extraction_layer;
            let hidden = self.model.forward(&input_ids, target_layer)?;
            
            // 4. Mean pooling sulla sequenza: [batch, seq, hidden] -> [batch, hidden]
            let last_hidden = hidden.mean(1)?;
            let last_hidden_vec = last_hidden.flatten_all()?.to_vec1::<f32>()?;
            
            println!("[Candle] Hidden states estratti: {} dim", last_hidden_vec.len());
            
            // 5. Logits (per generazione futura - TODO)
            let logits = vec![];
            
            Ok(ForwardOutput {
                logits,
                hidden_states: vec![last_hidden_vec.clone()],
                last_hidden: last_hidden_vec,
            })
        }
        
        /// Costruisce il vocabolario consentito dal lessico di Prometeo
        pub fn build_allowed_vocabulary(&mut self, engine: &super::PrometeoTopologyEngine) -> Result<usize> {
            self.allowed_tokens.clear();
            
            println!("[Candle] Costruzione vocabolario consentito...");
            
            // Itera su tutte le parole del lessico
            let mut words_mapped = 0;
            let mut words_failed = 0;
            
            for (word, _) in engine.lexicon.patterns_iter() {
                // Tokenizza la parola
                match self.tokenizer.encode(word.as_str(), false) {
                    Ok(encoding) => {
                        let token_ids = encoding.get_ids();
                        
                        // Aggiungi tutti i token della parola
                        for &token_id in token_ids {
                            self.allowed_tokens.insert(token_id);
                        }
                        words_mapped += 1;
                    }
                    Err(_) => {
                        words_failed += 1;
                    }
                }
            }
            
            // Aggiungi token speciali (spazi, punteggiatura, EOS)
            let special_tokens = vec![
                " ", ".", ",", "!", "?", ":", ";", "-", "'", "\"",
                "\n", "</s>", "<|endoftext|>", "<|im_end|>",
            ];
            
            for special in special_tokens {
                if let Ok(encoding) = self.tokenizer.encode(special, false) {
                    for token_id in encoding.get_ids() {
                        self.allowed_tokens.insert(*token_id);
                    }
                }
            }
            
            println!("[Candle] Vocabolario: {} parole mappate, {} fallite", 
                     words_mapped, words_failed);
            println!("[Candle] Token consentiti: {}", self.allowed_tokens.len());
            
            Ok(self.allowed_tokens.len())
        }
        
        /// Genera testo vincolato al lessico di Prometeo
        pub fn generate_inhabited(&mut self, 
                                  engine: &super::PrometeoTopologyEngine) -> Result<super::InhabitedResponse> {
            // 1. Costruisci context dalla memoria di Prometeo
            let context = self.build_prometeo_context(engine);
            
            // 2. Forward pass
            let output = self.forward_with_states(&context)?;
            
            // 3. Proietta sulle 8 dimensioni
            let field_8d = if !output.last_hidden.is_empty() {
                self.project_to_8d(&output.last_hidden)?
            } else {
                [0.5; 8]  // fallback
            };
            
            // 4. Sample con vincoli lessicali
            let generated_tokens = self.sample_constrained(&output.logits)?;
            
            // 5. Decodifica
            let text = self.tokenizer
                .decode(&generated_tokens, true)
                .map_err(|e| anyhow::anyhow!("Decode error: {}", e))?;
            
            // 6. Analizza parole conosciute vs sconosciute
            let (known, unknown) = self.classify_words(&text, engine);
            
            Ok(super::InhabitedResponse {
                field_signature: field_8d,
                text,
                layer_activations: Some(output.hidden_states),
                dominant_fractal: self.infer_fractal(&field_8d),
                known_words: known,
                unknown_words: unknown,
            })
        }
        
        fn build_prometeo_context(&self, engine: &super::PrometeoTopologyEngine) -> String {
            // Identico a build_context_from_prometeo sopra
            format!("STATO PROMETEO: {:?}", engine.field_sig())
        }
        
        fn project_to_8d(&self, hidden: &[f32]) -> Result<[f32; 8]> {
            let hidden_array = ndarray::Array1::from_vec(hidden.to_vec());
            let projected = self.projection.t().dot(&hidden_array);
            
            let mut result = [0.0f32; 8];
            for i in 0..8 {
                result[i] = ((projected[i].tanh() + 1.0) / 2.0).clamp(0.0, 1.0);
            }
            
            Ok(result)
        }
        
        fn sample_constrained(&self, logits: &[f32]) -> Result<Vec<u32>> {
            if logits.is_empty() {
                return Ok(vec![]);
            }
            
            // 1. Maschera i token non consentiti
            let mut masked_logits = logits.to_vec();
            for (token_id, logit) in masked_logits.iter_mut().enumerate() {
                if !self.allowed_tokens.contains(&(token_id as u32)) {
                    *logit = f32::NEG_INFINITY;
                }
            }
            
            // 2. Applica temperatura
            let temp = self.config.temperature;
            for logit in &mut masked_logits {
                if logit.is_finite() {
                    *logit /= temp;
                }
            }
            
            // 3. Softmax
            let max_logit = masked_logits.iter()
                .filter(|x| x.is_finite())
                .copied()
                .fold(f32::NEG_INFINITY, f32::max);
            
            let mut probs: Vec<f32> = masked_logits.iter()
                .map(|&l| if l.is_finite() { (l - max_logit).exp() } else { 0.0 })
                .collect();
            
            let sum: f32 = probs.iter().sum();
            if sum <= 0.0 {
                anyhow::bail!("Nessun token valido dopo mascheramento");
            }
            
            for p in &mut probs {
                *p /= sum;
            }
            
            // 4. Top-p (nucleus) sampling
            let mut indexed_probs: Vec<(usize, f32)> = probs.iter()
                .enumerate()
                .filter(|(_, &p)| p > 0.0)
                .map(|(i, &p)| (i, p))
                .collect();
            
            indexed_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            let mut cumsum = 0.0;
            let mut nucleus = Vec::new();
            for (idx, prob) in indexed_probs {
                cumsum += prob;
                nucleus.push((idx, prob));
                if cumsum >= self.config.top_p {
                    break;
                }
            }
            
            // 5. Sample dal nucleus
            let nucleus_sum: f32 = nucleus.iter().map(|(_, p)| p).sum();
            let mut rng = rand::thread_rng();
            let mut roll: f32 = rand::Rng::gen_range(&mut rng, 0.0..nucleus_sum);
            
            for (idx, prob) in &nucleus {
                roll -= prob;
                if roll <= 0.0 {
                    return Ok(vec![*idx as u32]);
                }
            }
            
            // Fallback: primo token del nucleus
            Ok(vec![nucleus[0].0 as u32])
        }
        
        fn classify_words(&self, text: &str, engine: &super::PrometeoTopologyEngine) 
            -> (Vec<String>, Vec<String>) {
            let words: Vec<_> = text.split_whitespace()
                .map(|w| crate::topology::lexicon::clean_token(w))
                .flatten()
                .collect();
            
            let mut known = vec![];
            let mut unknown = vec![];
            
            for word in words {
                if engine.lexicon.knows(&word) {
                    known.push(word);
                } else {
                    unknown.push(word);
                }
            }
            
            (known, unknown)
        }
        
        fn infer_fractal(&self, _sig: &[f32; 8]) -> Option<FractalId> {
            // Trova il frattale più vicino alla firma 8D
            // TODO: usa fractal.rs per calcolare distanze
            None
        }
    }
    
    struct ForwardOutput {
        logits: Vec<f32>,
        hidden_states: Vec<Vec<f32>>,
        last_hidden: Vec<f32>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_substrate_creation() {
        let config = SubstrateConfig::default();
        let substrate = LLMSubstrate::new(config);
        assert!(!substrate.is_calibrated);
    }
    
    #[test]
    fn test_projection_dimensions() {
        let substrate = LLMSubstrate::new(SubstrateConfig::default());
        assert_eq!(substrate.projection_matrix.ncols(), 8);
    }
}
